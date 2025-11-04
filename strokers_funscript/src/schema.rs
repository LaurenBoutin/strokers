use std::cmp::max;

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;
use tracing::warn;

use strokers_core::AxisKind;

/// A funscript is a JSON-encoded document that describes how one axis moves throughout time.
///
/// You should call [`Self::fixup`] on this afterwards if you want to interpret it.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Funscript {
    /// List of actions **sorted by timestamp order**
    /// (or at least I am claiming this needs to be sorted until proven otherwise)
    pub actions: Vec<FunscriptAction>,

    /// I imagine this is whether the movement is inverted or not.
    #[serde(default)]
    pub inverted: bool,

    /// The maximum value of `pos` in an action.
    /// Typical value seems to be 100.
    /// If not set, I guess we should compute it to be the max position, or 100, whichever is highest.
    #[serde(default)]
    pub range: u32,

    /// Multiscript can contains all axes in the same file under the `axes` field
    #[serde(default)]
    pub axes: Vec<FunscriptAxe>,

    /// Keys that we don't know about or don't care to implement right now.
    /// This just ensures they get preserved if we re-emit the file.
    #[serde(flatten)]
    pub unknown: serde_json::Value,
}

impl Funscript {
    /// Applies fixups to the funscript
    ///
    /// Current fixups:
    /// - populate a value for `range` if it is unset (zero)
    pub fn fixup(&mut self) {
        if self.range == 0 {
            // If the range isn't set, then set it to 100 or whatever the maximum value is in the file.
            self.range = max(
                self.actions
                    .iter()
                    .map(|action| action.pos)
                    .max()
                    .unwrap_or(100),
                100,
            );
        }
    }

    /// Return a funscript for the extra axis defined in the file
    pub fn get_axes_funscript(&mut self) -> BTreeMap<AxisKind, Funscript> {
        self.axes
            .iter()
            .filter_map(|axe| match AxisKind::try_from_tcode_axis_name(&axe.id) {
                Ok(axis_kind) => {
                    let script = Funscript {
                        actions: axe.actions.clone(),
                        inverted: self.inverted,
                        range: self.range,
                        axes: Vec::new(),
                        unknown: json!(null),
                    };
                    Some((axis_kind, script))
                }
                Err(err) => {
                    warn!("{err}");
                    None
                }
            })
            .collect::<BTreeMap<AxisKind, Funscript>>()
    }
}

/// One datapoint on the 'curve' that the funscript represents
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FunscriptAction {
    /// Timestamp in milliseconds relative to the start of the video
    pub at: u32,

    /// The position of the movement at this point in time
    pub pos: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FunscriptAxe {
    // Name of the axe
    pub id: String,
    /// List of actions **sorted by timestamp order**
    /// (or at least I am claiming this needs to be sorted until proven otherwise)
    pub actions: Vec<FunscriptAction>,
}
