use std::collections::VecDeque;

use crate::client::pathfind::context::{MoveRecord};
use crate::client::physics::Line;
use crate::client::state::global::GlobalState;
use crate::client::state::local::LocalState;
use crate::types::{Direction, Displacement, Location};
use crate::client::pathfind::incremental::PathResult;
use crate::client::physics::speed::Speed;

const PROGRESS_THRESHOLD: f64 = 0.2;
const PROGRESS_THRESHOLD_Y: f64 = 1.3;

#[derive(Eq, PartialEq)]
pub enum FollowResult {
    Failed,
    InProgress,
    Success,
}

#[derive(Debug)]
pub struct Follower {
    xs: VecDeque<Location>,
    initial: usize,
    ticks: usize,
    complete: bool,
    should_recalc: bool
}

impl Follower {
    pub fn new(path_result: PathResult<MoveRecord>) -> Option<Follower> {
        let path = path_result.value;
        if path.len() <= 1 { return None; }

        let initial = path.len();
        let xs = path.into_iter().map(|ctx| {
            let loc = ctx.state.location;
            loc.center_bottom()
        }).collect();

        Some(Follower {
            xs,
            initial,
            ticks: 0,
            complete: path_result.complete,
            should_recalc: false,
        })
    }

    fn next(&mut self) {
        self.xs.pop_front();
        self.ticks = 0;
    }

    pub fn should_recalc(&mut self) -> bool {

        // we should only recalc if this is not complete
        if self.complete {
            return false;
        }
        // we should only return once
        if self.should_recalc {
            return false;
        }
        let recalc = self.xs.len() * 2 < self.initial;
        self.should_recalc = recalc;
        recalc
    }

    pub fn follow(&mut self, local: &mut LocalState, global: &mut GlobalState) -> FollowResult {
        self.ticks += 1;

        // more than 7 seconds on same block => failed
        if self.ticks >= 20 * 7 {
            return FollowResult::Failed;
        }

        let next = self.xs.front();

        let next = match next {
            None => return if self.complete {FollowResult::Success} else {FollowResult::Failed},
            Some(next) => *next
        };

        let current = local.physics.location();
        let displacement = next - current;

        let mag2_horizontal = Displacement::new(displacement.dx, 0.0, displacement.dz).mag2();

        // sqrt(2) is 1.41 which is the distance from the center of a block to the next
        if mag2_horizontal > 1.6 * 1.6 {
            return FollowResult::Failed;
        }

        let res = if mag2_horizontal < PROGRESS_THRESHOLD * PROGRESS_THRESHOLD && displacement.dy.abs() < PROGRESS_THRESHOLD_Y {
            self.next();
            FollowResult::Success
        } else {
            FollowResult::InProgress
        };

        let mag2 = Displacement::new(displacement.dx, 0.0, displacement.dz).mag2();

        if mag2 < 0.01 * 0.01 {
            self.next();
            // want to avoid divide by 0 for direction
            return FollowResult::Success;
        }

        let dir = Direction::from(displacement);
        local.physics.look(dir);

        if displacement.dy > 0.0 {
            // we want to move vertically first (jump)
            local.physics.jump();
        }
        else if displacement.dy < 0.0 {
            // only will do anything if we are in water
            // local.physics.descend();
        }

        local.physics.line(Line::Forward);
        local.physics.speed(Speed::SPRINT);

        res
    }
}
