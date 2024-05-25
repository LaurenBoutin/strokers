use async_trait::async_trait;
use eyre::anyhow;
use strokers_core::{AxisDescriptor, AxisId, AxisKind, Movement, Stroker};
use tracing::{debug, error};

/// Does not connect to a real device; only emits log lines.
#[non_exhaustive]
pub struct DebugStroker {}

impl DebugStroker {
    pub fn new() -> DebugStroker {
        DebugStroker {}
    }
}

const AXES: &[(AxisId, AxisKind)] = &[
    (AxisId(1), AxisKind::Stroke),
    (AxisId(2), AxisKind::Surge),
    (AxisId(3), AxisKind::Sway),
    (AxisId(4), AxisKind::Twist),
    (AxisId(5), AxisKind::Roll),
    (AxisId(6), AxisKind::Pitch),
];

#[async_trait]
impl Stroker for DebugStroker {
    fn axes(&mut self) -> Vec<AxisDescriptor> {
        let result = AXES
            .into_iter()
            .cloned()
            .map(|(axis_id, axis_kind)| AxisDescriptor { axis_id, axis_kind })
            .collect();
        debug!("axes() = {result:?}");
        result
    }

    async fn stop(&mut self) -> eyre::Result<()> {
        debug!("stop()");
        Ok(())
    }

    async fn movement(&mut self, movement: Movement) -> eyre::Result<()> {
        match AXES.into_iter().find(|(id, _)| *id == movement.axis()) {
            Some((_, axis_kind)) => {
                debug!(
                    "movement({axis_kind:?}={:?} to {:.4} in {} ms)",
                    movement.axis(),
                    movement.target(),
                    movement.ramp_time_milliseconds()
                );
                Ok(())
            }
            None => {
                error!(
                    "movement(BAD AXIS={:?} to {:.4} in {} ms)",
                    movement.axis(),
                    movement.target(),
                    movement.ramp_time_milliseconds()
                );
                Err(anyhow!("Invalid axis"))
            }
        }
    }

    fn description(&mut self) -> eyre::Result<Option<String>> {
        let result = "DebugStroker".to_owned();
        debug!("description() = {result:?}");
        Ok(Some(result))
    }
}
