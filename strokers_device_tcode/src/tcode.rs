use std::{cmp::min, collections::BTreeMap, str::FromStr};

use eyre::{bail, Context, ContextCompat};
use strokers_core::{AxisId, Movement};

/// Converts a [`Movement`] to a T-Code command
/// Axis IDs are converted to T-Code axis names by using the `axis_map`.
pub(crate) fn movement_to_tcode(
    axis_map: &BTreeMap<AxisId, DiscoveredAxisInfo>,
    movement: &Movement,
) -> eyre::Result<String> {
    let axis_name = &axis_map
        .get(&movement.axis())
        .with_context(|| format!("no such axis: {:?}", movement.axis()))?
        .tcode_axis_name;

    assert!(movement.target() >= 0.0);
    let target_int = min((movement.target() * 10000.0) as u16, 9999);

    let ramp_int = movement.ramp_time_milliseconds();

    Ok(format!("{axis_name}{target_int:04}I{ramp_int:04}"))
}

/// The parsed format of a D2 response line.
/// e.g. each one of these is a line in the response
/// ```
/// L0 0 9999 Up
/// R0 0 9999 Twist
/// R1 0 9999 Roll
/// R2 0 9999 Pitch
/// V0 0 9999 Vibe1
/// V1 0 9999 Vibe2
/// A0 0 9999 Valve
/// A1 0 9999 Suck
/// ```
pub(crate) struct DiscoveredAxisInfo {
    pub tcode_axis_name: String,
    pub preferred_min: u16,
    pub preferred_max: u16,
    pub identified_name: String,
}

impl FromStr for DiscoveredAxisInfo {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let splits: Vec<&str> = s.split(" ").collect();
        if splits.len() < 4 {
            bail!("Not enough splits; should have 4 fields");
        }
        let tcode_axis_name = splits[0].to_owned();
        let preferred_min: u16 = splits[1]
            .parse()
            .context("failed to parse preferred_min!")?;
        let preferred_max: u16 = splits[2]
            .parse()
            .context("failed to parse preferred_max!")?;
        let identified_name = splits[3].to_owned();

        Ok(DiscoveredAxisInfo {
            tcode_axis_name,
            preferred_min,
            preferred_max,
            identified_name,
        })
    }
}

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;

    use strokers_core::{AxisId, Movement};

    use crate::tcode::{movement_to_tcode, DiscoveredAxisInfo};

    #[test]
    fn test_movement_to_tcode() {
        let mut axis_map = BTreeMap::new();
        axis_map.insert(
            AxisId(1),
            DiscoveredAxisInfo {
                tcode_axis_name: "L0".to_owned(),
                preferred_min: 0,
                preferred_max: 9999,
                identified_name: "Up".to_owned(),
            },
        );
        assert_eq!(
            movement_to_tcode(&axis_map, &Movement::new(AxisId(1), 0.75, 42).unwrap()).unwrap(),
            "L07500I0042"
        );
    }
}
