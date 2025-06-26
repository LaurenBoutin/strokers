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

    let inverted = funscript.inverted;

    for action in &funscript.actions {
        let FunscriptAction { at, pos } = *action;

        let norm_pos = if inverted {
            1.0 - (pos as f64 / 100.0)
        } else {
            pos as f64 / 100.0
        } as f32;

        out.push(NormalisedAction { at, norm_pos });
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_normalised_from_funscript() {
        macro_rules! genFunscriptTest {
            ($inverted:expr, $range:expr, $pos:expr, $norm_pos_expected:expr) => {
                FunscriptTest {
                    funscript: Funscript {
                        inverted: $inverted,
                        range: $range,
                        actions: Vec::from([FunscriptAction { at: 0, pos: $pos }]),
                        axes: Vec::new(),
                        unknown: json!(null),
                    },
                    norm_pos_expected: $norm_pos_expected,
                }
            };
        }

        struct FunscriptTest {
            funscript: Funscript,
            norm_pos_expected: f32,
        }

        let tests = Vec::from([
            // Inverted false and range 100
            genFunscriptTest!(false, 100, 0, 0.0),
            genFunscriptTest!(false, 100, 100, 1.0),
            genFunscriptTest!(false, 100, 50, 0.5),
            genFunscriptTest!(false, 100, 25, 0.25),
            // Inverted true and range 100
            genFunscriptTest!(true, 100, 0, 1.0),
            genFunscriptTest!(true, 100, 100, 0.0),
            genFunscriptTest!(true, 100, 50, 0.5),
            genFunscriptTest!(true, 100, 25, 0.75),
            // Inverted false and range 90
            genFunscriptTest!(false, 90, 0, 0.0),
            genFunscriptTest!(false, 90, 100, 1.0),
            genFunscriptTest!(false, 90, 50, 0.5),
            genFunscriptTest!(false, 90, 25, 0.25),
            // Inverted true and range 90
            genFunscriptTest!(true, 90, 0, 1.0),
            genFunscriptTest!(true, 90, 100, 0.0),
            genFunscriptTest!(true, 90, 50, 0.5),
            genFunscriptTest!(true, 90, 25, 0.75),
        ]);

        for test in tests.iter() {
            let result = normalised_from_funscript(&test.funscript);
            assert_eq!(test.norm_pos_expected, result[0].norm_pos);
        }
    }
}
