use std::{path::PathBuf, sync::Arc, time::Duration};

use eyre::{bail, Context, ContextCompat};
use flume::{Receiver, Sender};
use mpv_client::{osd, Client};
use strokers::{
    config::LimitsConfig,
    core::{AxisKind, Stroker},
};
use strokers_funscript::{
    processing::{normalised_from_funscript, NormalisedAction},
    schema::Funscript,
    search_path::scan_for_funscripts,
};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};

use crate::{
    keybindings::{AxisLimitChangeCommand, KeyCommand},
    playstate::{AxisLimiter, AxisPlaystate, Playstate},
};

#[derive(Clone, Debug)]
pub enum PlaythreadMessage {
    /// A new video was loaded
    /// - Unload all current funscripts
    /// - Search for new funscripts
    VideoStarting { video_path: PathBuf },
    /// Use the given loaded funscript
    UseFunscript {
        axis_kind: AxisKind,
        normalised_actions: Vec<NormalisedAction>,
    },
    /// The video playback time has updated in a sudden way
    Seek { now_millis: u32 },
    /// The video playback time has updated
    TimeChange { now_millis: u32 },
    /// The video pause state has updated
    PauseChange { paused: bool },
    /// MPV is shutting down so we should too
    Shutdown {},
    /// A key command was triggered
    KeyCommand(KeyCommand),
}

pub(crate) async fn playtask(
    mut stroker: impl Stroker,
    config: strokers::config::RootConfig,
    rx: Receiver<PlaythreadMessage>,
    tx: Sender<PlaythreadMessage>,
    mut weak_client: Client,
) -> eyre::Result<()> {
    let mut paused = false;
    let axes = stroker.axes();
    let mut playstate = Playstate::default();

    let mut funscript_load_ctoken: Option<CancellationToken> = None;

    while let Ok(msg) = rx.recv_async().await {
        match msg {
            PlaythreadMessage::VideoStarting { video_path } => {
                debug!("VideoStarting: {video_path:?}");
                let video_dir = video_path
                    .parent()
                    .context("video has no parent")?
                    .to_owned();
                let video_filename = video_path
                    .file_name()
                    .context("video has no filename")?
                    .to_str()
                    .context("video filename is not UTF-8")?
                    .to_owned();

                if let Some(ctoken) = funscript_load_ctoken.take() {
                    ctoken.cancel();
                }

                let new_ctoken = CancellationToken::new();
                funscript_load_ctoken = Some(new_ctoken.clone());

                let tx = tx.clone();
                tokio::task::spawn(async move {
                    tokio::select! {
                        res = search_for_funscripts(video_dir, video_filename, tx) => {
                            if let Err(err) = res {
                                error!("failed to handle VideoLoaded: {err:?}");
                            }
                        }
                        _ = new_ctoken.cancelled() => {
                            info!("search_for_funscripts cancelled");
                        }
                    }
                });
            }
            PlaythreadMessage::UseFunscript {
                axis_kind,
                normalised_actions,
            } => {
                debug!(
                    "UseFunscript: {axis_kind:?} ({} actions)",
                    normalised_actions.len()
                );
                let Some(axis) = axes.iter().find(|axis| axis.axis_kind == axis_kind) else {
                    warn!("can't use loaded funscript for {axis_kind:?} because the stroker doesn't have an axis for it");
                    continue;
                };

                let limits = match config.limits.get(&axis.axis_kind) {
                    Some(limits) => limits,
                    None => {
                        warn!("Axis {:?} has no limits configured; using some very pessimistic/safe/boring ones!", axis.axis_kind);
                        &LimitsConfig {
                            speed: 0.25,
                            default_min: 0.4,
                            default_max: 0.6,
                        }
                    }
                };

                playstate.by_axis.insert(
                    axis.axis_id,
                    AxisPlaystate::new(
                        Arc::new(normalised_actions),
                        limits.speed,
                        limits.default_min,
                        limits.default_max,
                    ),
                );
            }
            PlaythreadMessage::Seek { now_millis } => {
                debug!("Seek: {now_millis}");
                for (&axis_id, axis_playstate) in playstate.by_axis.iter_mut() {
                    axis_playstate
                        .seek(now_millis, paused, axis_id, &mut stroker)
                        .await
                        .context("failed AP tick")?;
                }
            }
            PlaythreadMessage::TimeChange { now_millis } => {
                if paused {
                    continue;
                }
                for (&axis_id, axis_playstate) in playstate.by_axis.iter_mut() {
                    axis_playstate
                        .tick(now_millis, axis_id, &mut stroker)
                        .await
                        .context("failed AP tick")?;
                }
            }
            PlaythreadMessage::PauseChange { paused: new_paused } => {
                debug!("PauseChange: {paused}");
                paused = new_paused;
                if paused {
                    stroker
                        .stop()
                        .await
                        .context("failed to stop stroker upon pause")?;
                } else {
                    // TODO
                    debug!("unpaused but proper resume is not supported");
                }
            }
            PlaythreadMessage::Shutdown {} => {
                debug!("Shutdown");
                stroker
                    .stop()
                    .await
                    .context("failed to stop stroker upon shutdown")?;
                break;
            }
            PlaythreadMessage::KeyCommand(cmd) => match cmd {
                KeyCommand::AxisLimitChange(cmd) => {
                    let Some(axis) = axes.iter().find(|axis| axis.axis_kind == cmd.axis) else {
                        warn!("Can't change axis limits for {:?} as there is no corresponding stroker axis", cmd.axis);
                        continue;
                    };
                    let Some(axis) = playstate.by_axis.get_mut(&axis.axis_id) else {
                        warn!(
                            "Can't change axis limits for {:?} as the axis is not in use.",
                            cmd.axis
                        );
                        continue;
                    };

                    if let Err(err) = update_limits(&cmd, &mut axis.limiter) {
                        error!("Error updating axis limits for {:?}: {err:?}", cmd.axis);
                    }
                    if let Err(err) = osd!(
                        weak_client,
                        Duration::from_secs(1),
                        "Limits: {:.4} ≤ {:?} ≤ {:.4}",
                        axis.limiter.min,
                        cmd.axis,
                        axis.limiter.max
                    ) {
                        error!("Failed to display OSD: {err:?}");
                    }
                }
            },
        }
    }
    Ok(())
}

/// Updates an axis's limits.
/// There is nothing preventing max < min although both limits are prevented from going out of range.
/// We can cheekily call max < min a 'feature' to allow inverting the motion *cough cough*.
fn update_limits(cmd: &AxisLimitChangeCommand, limits: &mut AxisLimiter) -> eyre::Result<()> {
    fn update_axis(
        name: &str,
        by: &Option<f32>,
        new: &Option<f32>,
        target: &mut f32,
    ) -> eyre::Result<()> {
        match (by, new) {
            (Some(_), Some(_)) => {
                bail!("Conflicting axis_limit parameters for {name}");
            }
            (Some(by), None) => {
                *target = (*target + by).clamp(0.0, 1.0);
            }
            (None, Some(new)) => {
                if *new < 0.0 || 1.0 < *new {
                    bail!("Can't set limit to {new:?} as that's out of range!");
                }
                *target = *new;
            }
            (None, None) => {
                // nop
            }
        }
        Ok(())
    }

    update_axis("min", &cmd.min_by, &cmd.min_new, &mut limits.min)?;
    update_axis("max", &cmd.max_by, &cmd.max_new, &mut limits.max)?;
    Ok(())
}

/// Given that the video has loaded, search for appropriate funscripts
///
/// TODO Currently this only searches for and loads 'main' cluster funscripts;
/// we should expand this in the future somehow.
async fn search_for_funscripts(
    video_dir: PathBuf,
    video_filename: String,
    tx: Sender<PlaythreadMessage>,
) -> eyre::Result<()> {
    let mut read_dir = tokio::fs::read_dir(&video_dir)
        .await
        .context("can't read")?;

    let mut filenames_in_dir: Vec<String> = Vec::new();
    while let Some(dir_entry) = read_dir
        .next_entry()
        .await
        .context("failed to read next directory entry")?
    {
        let file_type = dir_entry
            .file_type()
            .await
            .context("can't probe type of file")?;
        if !(file_type.is_file() || file_type.is_symlink()) {
            continue;
        }
        let raw_filename = dir_entry.file_name();
        let Some(filename) = raw_filename.to_str() else {
            warn!("skipping potential funscript file because it has a non-UTF8 filename");
            continue;
        };

        filenames_in_dir.push(filename.to_owned());
    }

    let scan = scan_for_funscripts(&filenames_in_dir, &video_filename)
        .context("failed funscript scan from list of filenames")?;

    for (&axis_kind, funscript_filename) in &scan.main.scripts {
        let funscript_path = video_dir.join(funscript_filename);
        let funscript_contents = tokio::fs::read(funscript_path)
            .await
            .with_context(|| format!("failed to read {funscript_filename:?}"))?;
        let mut funscript: Funscript = serde_json::from_slice(&funscript_contents)
            .with_context(|| format!("failed to deserialise {funscript_filename:?}"))?;
        funscript.fixup();
        let normalised_actions = normalised_from_funscript(&funscript);

        if let Err(_) = tx
            .send_async(PlaythreadMessage::UseFunscript {
                axis_kind,
                normalised_actions,
            })
            .await
        {
            warn!("loaded funscript but failed to send to playtask");
        }
    }

    Ok(())
}
