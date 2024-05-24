use std::sync::Arc;

use crate::processing::NormalisedAction;

/// Tracker for playback of a funscript.
pub struct FunscriptPlaystate {
    /// The normalised actions from the funscript
    normalised_actions: Arc<Vec<NormalisedAction>>,

    /// The index of the next action. Can point beyond the array to mean the playback is finished.
    next_index: usize,

    /// Time at which the next tick is due
    next_tick_at: Option<u32>,
}

impl FunscriptPlaystate {
    pub fn new(normalised_actions: Arc<Vec<NormalisedAction>>) -> FunscriptPlaystate {
        FunscriptPlaystate {
            normalised_actions,
            next_index: 0,
            next_tick_at: Some(0),
        }
    }

    /// Seek in the stream to a given time in milliseconds.
    pub fn seek(&mut self, time_milliseconds: u32) {
        // always tick immediately so that we update our position when we get the chance
        self.next_tick_at = Some(time_milliseconds);

        self.next_index = match self
            .normalised_actions
            .binary_search_by_key(&(time_milliseconds + 1), |action| action.at)
        {
            Ok(idx) => idx,
            Err(idx) => idx,
        };
    }

    /// Inform the playstate about the current time and see if there is an action to be performed
    pub fn tick(&mut self, time_milliseconds: u32) -> Option<NormalisedAction> {
        let Some(next_tick_at) = self.next_tick_at else {
            return None;
        };

        if time_milliseconds < next_tick_at {
            return None;
        }

        if self.next_index >= self.normalised_actions.len() {
            return None;
        }

        let next_action = self.normalised_actions[self.next_index];
        self.next_index += 1;

        self.next_tick_at = Some(next_action.at);

        Some(next_action)
    }
}
