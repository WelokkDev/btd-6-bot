//! A bot that plays Bloons TD 6 by looking at the screen.
//!
//! The design hangs off four traits, each with a real implementation and a fake
//! one. The fakes are what let the project be developed and tested without
//! launching the game.
//!
//! | Trait | Real | Fake |
//! |---|---|---|
//! | [`capture::Capture`] | [`capture::XcapCapture`] | [`capture::ReplayCapture`] |
//! | [`perception::Perceive`] | [`perception::HudPerceiver`] | — |
//! | [`brain::Decide`] | [`brain::RoundWatchBrain`] | — |
//! | [`actuator::Actuate`] | [`actuator::EnigoActuator`] | [`actuator::DryRunActuator`] |
//!
//! Layering rule: [`domain`] depends on nothing else here. If `xcap` or `enigo`
//! ever appears in it, the brain stops being testable without a game.

pub mod actuator;
pub mod brain;
pub mod capture;
pub mod config;
pub mod domain;
pub mod engine;
pub mod error;
pub mod geom;
pub mod perception;
pub mod platform;
