use std::path::PathBuf;

use eyre::Context;
use flume::{Receiver, Sender};
use mpv_client::{mpv_handle, Event, Handle};
use playthread::PlaythreadMessage;
use tracing::{error, info};
use tracing_subscriber::{fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt};

pub(crate) mod playstate;
mod playthread;

const PROP_TIME: &str = "time-pos/full";
const REPLY_TIME: u64 = 1;
const PROP_PAUSE: &str = "pause";
const REPLY_PAUSE: u64 = 2;

const PROP_PATH: &str = "path";

#[no_mangle]
extern "C" fn mpv_open_cplugin(handle: *mut mpv_handle) -> std::os::raw::c_int {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "strokers=debug,info".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_span_events(FmtSpan::CLOSE))
        .init();
    let client = Handle::from_ptr(handle);

    info!("strokers plugin for MPV ({}) is loaded!", client.name());

    let (tx, rx) = flume::bounded(4);
    let tx2 = tx.clone();

    std::thread::spawn(move || {
        if let Err(err) = start_playtask(rx, tx2) {
            error!("playtask failed: {err:?}")
        }
    });

    // Properties we care about:
    // - working_directory (or since we run in-process, we can probably just ignore that...)
    // - path (path to media, could be relative)
    // - time-pos/full (current playback position in milliseconds)
    //   - playback-time/full is similar but clamped to the duration of the file. I don't think we want that
    // - pause

    if let Err(err) = client.observe_property::<f64>(REPLY_TIME, PROP_TIME) {
        error!("can't register for {PROP_TIME}: {err:?}");
    }
    if let Err(err) = client.observe_property::<bool>(REPLY_PAUSE, PROP_PAUSE) {
        error!("can't register for {PROP_PAUSE}: {err:?}");
    }

    loop {
        match client.wait_event(-1.) {
            Event::Shutdown => {
                let _ = tx.send(PlaythreadMessage::Shutdown {});
                return 0;
            }
            Event::StartFile(_) => match client.get_property::<String>(PROP_PATH) {
                Ok(new_path) => {
                    info!("New video starting: {new_path:?}");
                    if let Err(_) = tx.send(PlaythreadMessage::VideoStarting {
                        video_path: PathBuf::from(new_path),
                    }) {
                        error!("New video loaded but can't send notification to playtask.")
                    }
                }
                Err(err) => {
                    error!("New video starting but failed to get {PROP_PATH}: {err:?}");
                }
            },
            Event::PropertyChange(REPLY_TIME, time_prop) => {
                let Some(time) = time_prop.data::<f64>() else {
                    error!("On change, can't read {PROP_TIME} as f64");
                    continue;
                };
                let Ok(time_millis_u32): Result<u32, _> = ((time * 1000.0) as i64).try_into()
                else {
                    continue;
                };
                let _ = tx.try_send(PlaythreadMessage::TimeChange {
                    now_millis: time_millis_u32,
                });
            }
            Event::PropertyChange(REPLY_PAUSE, pause_prop) => {
                let Some(paused) = pause_prop.data::<bool>() else {
                    error!("can't read {PROP_PAUSE} as bool");
                    continue;
                };
                if let Err(_) = tx.send(PlaythreadMessage::PauseChange { paused }) {
                    error!("Couldn't send pause change status to playtask.");
                }
            }
            Event::Seek => {
                let Ok(time) = client.get_property::<f64>(PROP_TIME) else {
                    error!("On seek, can't fetch {PROP_TIME} as f64");
                    continue;
                };
                let Ok(time_millis_u32): Result<u32, _> = ((time * 1000.0) as i64).try_into()
                else {
                    continue;
                };
                if let Err(_) = tx.send(PlaythreadMessage::Seek {
                    now_millis: time_millis_u32,
                }) {
                    error!("Couldn't send seek event to playtask.");
                }
            }
            event => {
                println!("Got event: {}", event);
            }
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn start_playtask(
    rx: Receiver<PlaythreadMessage>,
    tx: Sender<PlaythreadMessage>,
) -> eyre::Result<()> {
    let config = strokers::load_config()
        .await
        .context("failed to load Strokers configuration")?;
    let stroker = strokers::open_stroker(&config.stroker)
        .await
        .context("failed to connect to Stroker")?;
    playthread::playtask(stroker, config, rx, tx).await?;
    Ok(())
}
