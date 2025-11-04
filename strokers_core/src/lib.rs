use async_trait::async_trait;
use eyre::{bail, Result};
use serde::{Deserialize, Serialize};

#[async_trait]
pub trait Stroker {
    fn axes(&mut self) -> Vec<AxisDescriptor>;

    /// Stop the stroker as soon as possible.
    async fn stop(&mut self) -> eyre::Result<()>;

    /// Perform a movement.
    async fn movement(&mut self, movement: Movement) -> eyre::Result<()>;

    /// Returns a human-readable description of the stroker device.
    /// Returns None if this device doesn't support that.
    fn description(&mut self) -> eyre::Result<Option<String>>;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AxisId(pub u32);

#[derive(Clone, Debug)]
pub struct AxisDescriptor {
    pub axis_id: AxisId,
    pub axis_kind: AxisKind,
}

/// The kind of axis
/// This includes special axes like vibration/lubricant
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum AxisKind {
    /// Up/Down (T-Code: `L0`)
    Stroke,
    /// Forward/Backward (T-Code: `L1`)
    Surge,
    /// Left/Right (T-Code: `L2`)
    Sway,
    /// (T-Code: `R0`)
    Twist,
    /// (T-Code: `R1`)
    Roll,
    /// (T-Code: `R2`)
    Pitch,
    /// (T-Code: `V0`)
    Vibration,
    /// (T-Code: `A0`)
    Valve,
    /// (T-Code: `A1`)
    Suction,
    /// (T-Code: `A2`)
    Lubricant,
}

impl AxisKind {
    pub fn try_from_tcode_axis_name(name: &str) -> Result<Self, eyre::Error> {
        match name {
            "L0" => Ok(AxisKind::Stroke),
            "L1" => Ok(AxisKind::Surge),
            "L2" => Ok(AxisKind::Sway),
            "R0" => Ok(AxisKind::Twist),
            "R1" => Ok(AxisKind::Roll),
            "R2" => Ok(AxisKind::Pitch),
            "V0" => Ok(AxisKind::Vibration),
            "A0" => Ok(AxisKind::Valve),
            "A1" => Ok(AxisKind::Suction),
            "A2" => Ok(AxisKind::Lubricant),
            other => {
                bail!("Unrecognised T-Code axis: {other:?}");
            }
        }
    }
}

/// Describes a desired movement.
#[derive(Clone, Debug)]
pub struct Movement {
    /// The ID of the axis to move
    axis: AxisId,
    /// The target position of the axis; normalised between 0.0 and 1.0
    target: f32,
    /// How long, in milliseconds, to take to ramp to this value.
    /// Between 0 and 9999999 (this upper limit is taken from the OSR2's implementation)
    ramp_time_milliseconds: u32,
}

impl Movement {
    /// Create a new movement
    ///
    /// - `axis`: The ID of the axis to move
    /// - `target`: The target position of the axis; normalised between 0.0 and 1.0
    /// - `ramp_time_milliseconds`: How long, in milliseconds, to take to ramp to this value.
    ///   Between 0 and 9999999 (this upper limit is taken from the OSR2's implementation)
    ///
    /// Returns the Movement or `None` if the specified parameters were not valid.
    pub fn new(axis: AxisId, target: f32, ramp_time_milliseconds: u32) -> Option<Movement> {
        const MAX_RAMP_TIME_MS: u32 = 9999999;
        if !(0.0..=1.0).contains(&target) {
            return None;
        };
        if !target.is_finite() {
            return None;
        }
        if ramp_time_milliseconds > MAX_RAMP_TIME_MS {
            return None;
        };

        Some(Movement {
            axis,
            target,
            ramp_time_milliseconds,
        })
    }

    pub fn axis(&self) -> AxisId {
        self.axis
    }

    pub fn target(&self) -> f32 {
        self.target
    }

    pub fn ramp_time_milliseconds(&self) -> u32 {
        self.ramp_time_milliseconds
    }
}
