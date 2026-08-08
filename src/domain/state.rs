//! What the bot believes about the world. Pure data.

use std::time::Duration;

/// A single observation of the game.
///
/// `Option` everywhere is deliberate: perception *fails*, and the type should say
/// so rather than inventing a zero. A brain that sees `cash: None` must wait, not
/// spend nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameState {
    pub screen: Screen,
    pub round: Option<Round>,
    pub cash: Option<u32>,
    pub lives: Option<u32>,
    /// True while a round is running (the ▶ button shows its active form).
    pub round_active: bool,
    /// Time since the engine started, taken from the frame's capture instant.
    ///
    /// Carried on the observation so `decide` stays pure (§3.3): a test builds a
    /// literal `Duration` rather than mocking a clock.
    pub observed_at: Duration,
}

impl GameState {
    /// An observation where nothing could be read.
    pub fn unknown(observed_at: Duration) -> Self {
        Self {
            screen: Screen::Unknown,
            round: None,
            cash: None,
            lives: None,
            round_active: false,
            observed_at,
        }
    }

    pub fn is_actionable(&self) -> bool {
        self.screen != Screen::Unknown
    }
}

/// BTD6 renders the round as `current/total` (e.g. `1/10`), so both halves come
/// out of one region.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Round {
    pub current: u32,
    pub total: Option<u32>,
}

impl Round {
    pub const fn new(current: u32, total: Option<u32>) -> Self {
        Self { current, total }
    }

    pub fn is_final(&self) -> bool {
        matches!(self.total, Some(t) if self.current >= t)
    }
}

/// Which screen we're looking at.
///
/// `Unknown` is a first-class variant, not an error: the brain must be able to
/// handle "I don't know where I am" by doing nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Screen {
    MainMenu,
    MapSelect,
    InGame,
    Victory,
    Defeat,
    Unknown,
}

impl Screen {
    pub const ALL: [Screen; 6] = [
        Screen::MainMenu,
        Screen::MapSelect,
        Screen::InGame,
        Screen::Victory,
        Screen::Defeat,
        Screen::Unknown,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Screen::MainMenu => "MainMenu",
            Screen::MapSelect => "MapSelect",
            Screen::InGame => "InGame",
            Screen::Victory => "Victory",
            Screen::Defeat => "Defeat",
            Screen::Unknown => "Unknown",
        }
    }
}

impl std::fmt::Display for Screen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_state_is_not_actionable() {
        assert!(!GameState::unknown(Duration::ZERO).is_actionable());
    }

    #[test]
    fn round_knows_when_it_is_the_last_one() {
        assert!(Round::new(10, Some(10)).is_final());
        assert!(Round::new(11, Some(10)).is_final());
        assert!(!Round::new(9, Some(10)).is_final());
        // Without a total we can never be sure, so we never claim to be.
        assert!(!Round::new(99, None).is_final());
    }
}
