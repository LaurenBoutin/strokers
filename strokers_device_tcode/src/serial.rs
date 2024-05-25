use std::{collections::BTreeMap, path::Path, str::FromStr, time::Duration};

use async_trait::async_trait;
use eyre::{Context, ContextCompat};
use futures_util::SinkExt;
use serial2_tokio::SerialPort;
use strokers_core::{AxisDescriptor, AxisId, AxisKind, Stroker};
use tokio::time::timeout;
use tokio_stream::StreamExt;
use tokio_util::codec::{Decoder, Framed, LinesCodec};
use tracing::{debug, error, warn};

use crate::tcode::{movement_to_tcode, DiscoveredAxisInfo};

pub struct SerialTCodeStroker {
    port: Framed<SerialPort, LinesCodec>,
    axis_map: BTreeMap<AxisId, DiscoveredAxisInfo>,
    description: String,
}

impl SerialTCodeStroker {
    pub async fn connect(
        serial_port_path: impl AsRef<Path>,
        baud: u32,
    ) -> eyre::Result<SerialTCodeStroker> {
        let serial_port =
            SerialPort::open(serial_port_path, baud).context("failed to open serial port")?;
        serial_port
            .discard_buffers()
            .context("failed to discard buffers")?;
        let mut line_codec = LinesCodec::new().framed(serial_port);

        debug!("attempting to identify T-Code device");

        line_codec
            .send("D0".to_owned())
            .await
            .context("failed to send D0 command")?;

        let d0_resp = line_codec
            .next()
            .await
            .context("end of stream on D0 command")?
            .context("failed to read D0 response")?;

        debug!("D0: {d0_resp}");

        line_codec
            .send("D1".to_owned())
            .await
            .context("failed to send D1 command")?;

        let d1_resp = line_codec
            .next()
            .await
            .context("end of stream on D1 command")?
            .context("failed to read D1 response")?;

        debug!("D1: {d1_resp}");

        line_codec
            .send("D2".to_owned())
            .await
            .context("failed to send D2 command")?;

        let mut axis_map = BTreeMap::new();

        let mut axis_id_generator = 0;
        while let Ok(Some(next)) = timeout(Duration::from_millis(200), line_codec.next()).await {
            let next_line = next.context("failed to read line")?;
            debug!("D2 response line: {next_line:?}");

            match DiscoveredAxisInfo::from_str(&next_line) {
                Ok(axis) => {
                    axis_map.insert(AxisId(axis_id_generator), axis);
                }
                Err(err) => {
                    error!(
                        "D2 axis description response {next_line:?} could not be parsed: {err:?}"
                    );
                }
            }
            axis_id_generator += 1;
        }

        Ok(SerialTCodeStroker {
            port: line_codec,
            axis_map,
            description: format!("{d0_resp} ({d1_resp})"),
        })
    }
}

#[async_trait]
impl Stroker for SerialTCodeStroker {
    fn axes(&mut self) -> Vec<strokers_core::AxisDescriptor> {
        let mut result = Vec::with_capacity(self.axis_map.len());
        for (&axis_id, axis) in &self.axis_map {
            let axis_kind = match axis.tcode_axis_name.as_str() {
                "L0" => AxisKind::Stroke,
                "L1" => AxisKind::Surge,
                "L2" => AxisKind::Sway,
                "R0" => AxisKind::Twist,
                "R1" => AxisKind::Roll,
                "R2" => AxisKind::Pitch,
                "V0" => AxisKind::Vibration,
                "A0" => AxisKind::Valve,
                "A1" => AxisKind::Suction,
                "A2" => AxisKind::Lubricant,
                other => {
                    warn!("Unrecognised T-Code axis: {other:?}; ignoring.");
                    continue;
                }
            };
            result.push(AxisDescriptor { axis_id, axis_kind });
        }
        result
    }

    async fn stop(&mut self) -> eyre::Result<()> {
        self.port
            .send("DSTOP".to_owned())
            .await
            .context("failed to send DSTOP command")
    }

    async fn movement(&mut self, movement: strokers_core::Movement) -> eyre::Result<()> {
        let tcode = movement_to_tcode(&self.axis_map, &movement)
            .with_context(|| format!("failed to encode T-Code for {movement:?}"))?;
        self.port
            .send(tcode)
            .await
            .context("failed to send T-Code command")
    }

    fn description(&mut self) -> eyre::Result<Option<String>> {
        Ok(Some(self.description.clone()))
    }
}
