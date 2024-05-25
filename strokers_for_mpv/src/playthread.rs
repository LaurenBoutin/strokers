use std::{path::PathBuf, sync::Arc};

use eyre::{Context, ContextCompat};
use flume::{Receiver, Sender};
use strokers::core::{AxisKind, Stroker};
use strokers_funscript::{
    processing::{normalised_from_funscript, NormalisedAction},
    schema::Funscript,
    search_path::scan_for_funscripts,
};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};

use crate::playstate::{AxisPlaystate, Playstate};

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
}

pub(crate) async fn playtask(
    mut stroker: impl Stroker,
    rx: Receiver<PlaythreadMessage>,
    tx: Sender<PlaythreadMessage>,
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

                playstate.by_axis.insert(
                    axis.axis_id,
                    AxisPlaystate::new(
                        Arc::new(normalised_actions),
                        axis.suggested_safe_speed_limit,
                        // TODO The min and max should not be hardcoded, plus they should be changeable on the fly
                        0.25,
                        0.75,
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
            }
        }
    }
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
        let funscript: Funscript = serde_json::from_slice(&funscript_contents)
            .with_context(|| format!("failed to deserialise {funscript_filename:?}"))?;
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
