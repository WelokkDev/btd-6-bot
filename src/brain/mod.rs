//! Deciding what to do.
//!
//! `Decide` is a pure function: no I/O, no clock, no `Result`. That is what
//! makes behaviour regression-testable — a test builds a `GameState` literal and
//! asserts the returned `Action`. `&mut self` is for the brain's own progress
//! state, never for I/O.

use crate::domain::{Action, GameState, Round};
use crate::geom::NormPoint;

pub trait Decide {
    fn decide(&mut self, state: &GameState) -> Action;

    fn name(&self) -> &'static str;
}

/// A deliberately dumb brain: watch the round number, emit one click when it
/// changes. It proves the loop closes and the trait boundaries are honest, and a
/// real scripted brain replaces it behind this same trait.
pub struct RoundWatchBrain {
    last_round: Option<u32>,
    poke_at: NormPoint,
    changes_seen: u64,
}

impl RoundWatchBrain {
    pub fn new(poke_at: NormPoint) -> Self {
        Self { last_round: None, poke_at, changes_seen: 0 }
    }

    pub fn changes_seen(&self) -> u64 {
        self.changes_seen
    }
}

impl Decide for RoundWatchBrain {
    fn decide(&mut self, state: &GameState) -> Action {
        // "I don't know where I am" must never produce an action.
        if !state.is_actionable() {
            return Action::Wait;
        }

        let Some(Round { current, .. }) = state.round else {
            // An unreadable round must not look like a change.
            return Action::Wait;
        };

        match self.last_round {
            Some(prev) if prev == current => Action::Wait,
            Some(prev) => {
                tracing::info!(from = prev, to = current, "round changed");
                self.last_round = Some(current);
                self.changes_seen += 1;
                Action::Click { at: self.poke_at }
            }
            // First sighting establishes a baseline; it is not a change.
            None => {
                tracing::info!(round = current, "first round observation");
                self.last_round = Some(current);
                Action::Wait
            }
        }
    }

    fn name(&self) -> &'static str {
        "round-watch"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Screen;
    use std::time::Duration;

    const POKE: NormPoint = NormPoint::new(0.5, 0.95);

    fn in_game(round: Option<u32>) -> GameState {
        GameState {
            screen: Screen::InGame,
            round: round.map(|r| Round::new(r, Some(40))),
            cash: Some(650),
            lives: Some(150),
            round_active: false,
            observed_at: Duration::ZERO,
        }
    }

    #[test]
    fn first_observation_is_a_baseline_not_a_change() {
        let mut b = RoundWatchBrain::new(POKE);
        assert_eq!(b.decide(&in_game(Some(1))), Action::Wait);
        assert_eq!(b.changes_seen(), 0);
    }

    #[test]
    fn a_round_change_emits_exactly_one_click() {
        let mut b = RoundWatchBrain::new(POKE);
        b.decide(&in_game(Some(1)));

        assert_eq!(b.decide(&in_game(Some(2))), Action::Click { at: POKE });
        // The same round must not fire again on subsequent ticks.
        assert_eq!(b.decide(&in_game(Some(2))), Action::Wait);
        assert_eq!(b.decide(&in_game(Some(2))), Action::Wait);
        assert_eq!(b.changes_seen(), 1);
    }

    #[test]
    fn an_unknown_screen_never_produces_an_action() {
        let mut b = RoundWatchBrain::new(POKE);
        let mut s = in_game(Some(5));
        s.screen = Screen::Unknown;
        assert_eq!(b.decide(&s), Action::Wait);
    }

    #[test]
    fn an_unreadable_round_holds_the_previous_belief() {
        let mut b = RoundWatchBrain::new(POKE);
        b.decide(&in_game(Some(7)));
        assert_eq!(b.decide(&in_game(None)), Action::Wait);
        assert_eq!(b.decide(&in_game(Some(7))), Action::Wait, "same round, no change");
        assert_eq!(b.changes_seen(), 0);
    }

    #[test]
    fn deciding_needs_no_game_screen_or_clock() {
        let mut b = RoundWatchBrain::new(POKE);
        let actions: Vec<Action> =
            [1, 1, 2, 3, 3].iter().map(|r| b.decide(&in_game(Some(*r)))).collect();
        assert_eq!(
            actions,
            vec![
                Action::Wait,
                Action::Wait,
                Action::Click { at: POKE },
                Action::Click { at: POKE },
                Action::Wait,
            ]
        );
    }
}
