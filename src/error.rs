//! Failure classification shared across layers.
//!
//! The engine reacts to these two classes very differently, so the distinction
//! belongs in the type system rather than in a comment.

/// How the engine should react to an error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// One bad frame, one failed read. Log it, skip the tick, keep going.
    Transient,
    /// The world is broken — window gone, config invalid. Stop cleanly.
    Fatal,
}

/// Implemented by every layer's error type so `engine` can branch on severity
/// without knowing which layer produced the error.
pub trait Classify {
    fn severity(&self) -> Severity;

    fn is_transient(&self) -> bool {
        self.severity() == Severity::Transient
    }

    fn is_fatal(&self) -> bool {
        self.severity() == Severity::Fatal
    }
}
