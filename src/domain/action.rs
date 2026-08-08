//! What the bot wants to do. Pure data — no coordinates are resolved here and no
//! I/O happens; the actuator owns all click mechanics (§3.4).

use crate::geom::NormPoint;

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// Do nothing this tick. The common case, and never an error.
    Wait,
    Click { at: NormPoint },
    /// Compound on purpose: the spike confirmed placement really is two clicks
    /// (sidebar select, then map click).
    PlaceTower { kind: TowerKind, at: NormPoint },
    Upgrade { target: usize, path: UpgradePath },
    /// Press the ▶ button to begin the next round.
    StartRound,
    /// Dismiss a modal (level-up, achievement) that's stealing clicks (§3.6).
    DismissPopup,
}

impl Action {
    /// Whether this action drives real input.
    pub fn is_noop(&self) -> bool {
        matches!(self, Action::Wait)
    }
}

/// Deliberately a closed set: a typo in a plan file should fail at load time, not
/// halfway through a run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum TowerKind {
    DartMonkey,
    BoomerangMonkey,
    BombShooter,
    TackShooter,
    IceMonkey,
    GlueGunner,
    SniperMonkey,
    MonkeySub,
    MonkeyBuccaneer,
    MonkeyAce,
    Druid,
    Alchemist,
    BananaFarm,
    SpikeFactory,
    MonkeyVillage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum UpgradePath {
    Top,
    Middle,
    Bottom,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_wait_is_a_noop() {
        assert!(Action::Wait.is_noop());
        assert!(!Action::StartRound.is_noop());
        assert!(!Action::Click { at: NormPoint::new(0.5, 0.5) }.is_noop());
    }
}
