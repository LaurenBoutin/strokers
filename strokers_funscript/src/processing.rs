use crate::schema::{Funscript, FunscriptAction};

/// A data point of where an axis should be at a given time, but normalised.
#[derive(Copy, Clone, Debug)]
pub struct NormalisedAction {
    /// Time in milliseconds since the start of the video
    pub at: u32,
    /// Where the axis should be, between 0.0 and 1.0 full scale
    pub norm_pos: f32,
}

/// Extract a list of normalised actions from a funscript.
///
/// These always go from 0.0 to 1.0 and don't have any 'inverted' flags to follow.
pub fn normalised_from_funscript(funscript: &Funscript) -> Vec<NormalisedAction> {
    let mut out = Vec::with_capacity(funscript.actions.len());

    let max_f64 = funscript.range as f64;
    let inverted = funscript.inverted;

    for action in &funscript.actions {
        let FunscriptAction { at, pos } = *action;

        let norm_pos = if inverted {
            max_f64 * (1.0 - (pos as f64 / max_f64))
        } else {
            pos as f64 / max_f64
        } as f32;

        out.push(NormalisedAction { at, norm_pos });
    }

    out
}
