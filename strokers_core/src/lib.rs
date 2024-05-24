use async_trait::async_trait;

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
    pub suggested_safe_speed_limit: f32,
}

/// The kind of axis
/// This includes special axes like vibration/lubricant
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

/// Describes a desired movement.
#[derive(Clone, Debug)]
pub struct Movement {
    /// The ID of the axis to move
    axis: AxisId,
    /// The target position of the axis; normalised between 0.0 and 1.0
    target: f32,
    /// How long, in milliseconds, to take to ramp to this value.
    /// Between 0 and 9999
    ramp_time_milliseconds: u16,
}

impl Movement {
    /// Create a new movement
    ///
    /// - `axis`: The ID of the axis to move
    /// - `target`: The target position of the axis; normalised between 0.0 and 1.0
    /// - `ramp_time_milliseconds`: How long, in milliseconds, to take to ramp to this value.
    ///   Between 0 and 9999
    ///
    /// Returns the Movement or `None` if the specified parameters were not valid.
    pub fn new(axis: AxisId, target: f32, ramp_time_milliseconds: u16) -> Option<Movement> {
        if target < 0.0 || target > 1.0 {
            return None;
        };
        if ramp_time_milliseconds > 9999 {
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

    pub fn ramp_time_milliseconds(&self) -> u16 {
        self.ramp_time_milliseconds
    }
}
