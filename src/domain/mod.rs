//! Pure data shared by every layer.
//!
//! Must not depend on `capture`, `perception`, or `actuator`. An `xcap` or
//! `enigo` import below this line means the brain is no longer unit-testable
//! without a running game.

pub mod action;
pub mod state;

pub use action::{Action, TowerKind, UpgradePath};
pub use state::{GameState, Round, Screen};
