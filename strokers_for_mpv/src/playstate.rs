use std::{
    collections::BTreeMap,
    sync::Arc,
    time::{Duration, Instant},
};

use eyre::{Context, ContextCompat};
use strokers::core::{AxisId, Movement, Stroker};
use strokers_funscript::{playstate::FunscriptPlaystate, processing::NormalisedAction};

#[derive(Default)]
pub(crate) struct Playstate {
    pub by_axis: BTreeMap<AxisId, AxisPlaystate>,
}

pub(crate) struct AxisPlaystate {
    funscript: FunscriptPlaystate,
    pub limiter: AxisLimiter,
}

impl AxisPlaystate {
    pub fn new(
        normalised_actions: Arc<Vec<NormalisedAction>>,
        speed_limit: f32,
        min: f32,
        max: f32,
    ) -> AxisPlaystate {
        AxisPlaystate {
            funscript: FunscriptPlaystate::new(normalised_actions),
            limiter: AxisLimiter::new(speed_limit, min, max),
        }
    }
    pub async fn tick(
        &mut self,
        now_millis: u32,
        axis_id: AxisId,
        stroker: &mut impl Stroker,
    ) -> eyre::Result<()> {
        if let Some(action) = self.funscript.tick(now_millis) {
            if action.at < now_millis {
                return Ok(());
            }
            let now = Instant::now();
            let (new_target, new_target_duration) =
                self.limiter
                    .limit_command(now, action.norm_pos, action.at - now_millis);
            self.limiter
                .notify_commanded(now, new_target, new_target_duration);
            stroker
                .movement(
                    Movement::new(axis_id, new_target, new_target_duration)
                    .with_context(|| {
                        format!("failed to construct movement from pos:{new_target}, {new_target_duration}ms")
                    })?,
                )
                .await
                .with_context(|| {
                    format!("failed to command movement from pos:{new_target}, {new_target_duration}ms")
                })?;
        }

        Ok(())
    }

    pub async fn seek(
        &mut self,
        now_millis: u32,
        paused: bool,
        axis_id: AxisId,
        stroker: &mut impl Stroker,
    ) -> eyre::Result<()> {
        self.funscript.seek(now_millis);

        if let Some(action) = self.funscript.tick(now_millis) {
            let now = Instant::now();

            // if the video is paused, give a long time to gradually move to the right position
            // that way we also likely avoid being speed limited.
            let orig_target_duration = if paused { 1000 } else { action.at - now_millis };

            let (new_target, new_target_duration) =
                self.limiter
                    .limit_command(now, action.norm_pos, orig_target_duration);
            self.limiter
                .notify_commanded(now, new_target, new_target_duration);
            stroker
                .movement(
                    Movement::new(axis_id, new_target, new_target_duration)
                    .with_context(|| {
                        format!("failed to construct seek movement from pos:{new_target}, {new_target_duration}ms")
                    })?,
                )
                .await
                .with_context(|| {
                    format!("failed to command seek movement from pos:{new_target}, {new_target_duration}ms")
                })?;
        }

        Ok(())
    }
}

/// Tracks current position and limits speed.
/// TODO should this move to `strokers` crate?
pub(crate) struct AxisLimiter {
    /// Maximum number of full-scale movements per second
    pub speed_limit: f32,
    /// Time of the last-issued command
    pub last_command_start_time: Instant,
    /// Estimated position at the start of the last-issued command
    pub last_command_start: f32,
    /// Target finishing time of the last-issued command
    pub last_command_target_time: Instant,
    /// Target finishing position of the last-issued command
    pub last_command_target: f32,
    /// The bottom limit of the axis
    pub min: f32,
    /// The top of the axis
    pub max: f32,
}

impl AxisLimiter {
    /// Estimates the position of the axis at the given current time.
    pub fn estimate_current_position(&self, now: Instant) -> f32 {
        if self.last_command_target_time < now {
            self.last_command_target
        } else if self.last_command_start_time < now {
            let proportion_complete = (now - self.last_command_start_time).as_secs_f64()
                / (self.last_command_target_time - self.last_command_start_time).as_secs_f64();
            self.last_command_start
                + (self.last_command_target - self.last_command_start) * proportion_complete as f32
        } else {
            self.last_command_start
        }
    }

    /// Postprocesses a proposed order to move to `target` in `duration_millis` ms
    /// and limits it according to the configured bottom, top and speed limits.
    pub fn limit_command(&self, now: Instant, target: f32, duration_millis: u32) -> (f32, u32) {
        let cur_pos = self.estimate_current_position(now);

        // Apply top and bottom limits
        let target = self.min + (self.max - self.min) * target;

        let delta = target - cur_pos;

        let speed_abs = delta.abs() / (duration_millis.max(1) as f32 * 0.001);

        if speed_abs < self.speed_limit {
            (target, duration_millis)
        } else {
            let proposed_delta = delta * (self.speed_limit / speed_abs);
            (cur_pos + proposed_delta, duration_millis)
        }
    }

    /// Updates the tracked state to reflect that we just commanded a move.
    pub fn notify_commanded(&mut self, now: Instant, target: f32, duration_millis: u32) {
        let start = self.estimate_current_position(now);
        let target_time = now + Duration::from_millis(duration_millis as u64);
        self.last_command_start = start;
        self.last_command_start_time = now;
        self.last_command_target = target;
        self.last_command_target_time = target_time;
    }

    pub fn new(speed_limit: f32, min: f32, max: f32) -> AxisLimiter {
        let now = Instant::now();
        AxisLimiter {
            speed_limit,
            last_command_start_time: now,
            last_command_start: 0.5,
            last_command_target_time: now,
            last_command_target: 0.5,
            min,
            max,
        }
    }
}
