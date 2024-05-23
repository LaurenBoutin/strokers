use std::time::Duration;

use eyre::Context;
use serial2_tokio::SerialPort;
use strokers_core::{Movement, Stroker};
use strokers_device_tcode::SerialTCodeStroker;
use tracing::info;
use tracing_subscriber::{fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main(flavor = "current_thread")]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "strokers=debug,info".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_span_events(FmtSpan::CLOSE))
        .init();

    info!("opening serial port");
    let serial_port =
        SerialPort::open("/dev/ttyUSB0", 115200).context("failed to open serial port")?;
    info!("connecting to t-code device");
    let mut stroker = SerialTCodeStroker::connect(serial_port)
        .await
        .context("failed to connect to serial port T-Code device")?;
    info!("connected to t-code device");

    for axis in stroker.axes() {
        info!("trying axis: {axis:?}");
        stroker
            .movement(Movement::new(axis.axis_id, 0.0, 2000).unwrap())
            .await?;
        tokio::time::sleep(Duration::from_secs(2)).await;
        stroker
            .movement(Movement::new(axis.axis_id, 1.0, 2000).unwrap())
            .await?;
        tokio::time::sleep(Duration::from_secs(2)).await;
    }

    Ok(())
}
