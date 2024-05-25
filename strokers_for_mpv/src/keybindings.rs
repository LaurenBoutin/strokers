use eyre::{bail, Context};
use serde::Deserialize;
use strokers::core::AxisKind;

#[derive(Clone, Debug)]
pub enum KeyCommand {
    AxisLimitChange(AxisLimitChangeCommand),
}

/// Changes the limit on an axis.
#[derive(Clone, Debug, Deserialize)]
pub struct AxisLimitChangeCommand {
    /// The axis to change the limit of
    pub axis: AxisKind,
    /// Change the axis minimum limit by the given amount.
    pub min_by: Option<f32>,
    /// Change the axis minimum limit to the given amount.
    pub min_new: Option<f32>,
    /// Change the axis maximum limit by the given amount.
    pub max_by: Option<f32>,
    /// Change the axis maximum limit to the given amount.
    pub max_new: Option<f32>,
}

pub fn parse_action(action: &str) -> eyre::Result<KeyCommand> {
    let (action_name, action_args_qs) = action.split_once(' ').unwrap_or((&action, ""));

    match action_name {
        "axis_limit" => {
            let cmd =
                serde_qs::from_str(action_args_qs).context("failed to parse axis_limit cmd")?;
            Ok(KeyCommand::AxisLimitChange(cmd))
        }
        _ => {
            bail!("unknown action: {action_name:?}");
        }
    }
}
