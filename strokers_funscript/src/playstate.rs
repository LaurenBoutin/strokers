use std::sync::Arc;

use tracing::debug;

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
        let idx_old = self.next_index;
        // always tick immediately so that we update our position when we get the chance
        self.next_tick_at = Some(time_milliseconds);

        self.next_index = match self
            .normalised_actions
            .binary_search_by_key(&(time_milliseconds + 1), |action| action.at)
        {
            Ok(idx) => idx,
            Err(idx) => idx,
        };

        let idx_new = self.next_index;
        let idx_1 = idx_new - 1;
        let ele_1 = self.normalised_actions.get(idx_1);
        let ele_2 = self.normalised_actions.get(idx_new);
        let idx_3 = idx_new + 1;
        let ele_3 = self.normalised_actions.get(idx_3);
        debug!("sought from idx{idx_old} to idx{idx_new} ({idx_1}={ele_1:?}, {idx_new}={ele_2:?}, {idx_3}={ele_3:?})");
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
