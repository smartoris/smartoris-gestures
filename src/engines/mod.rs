//! Gesture engines.
//!
//! See [`GestureEngine`] for details.

mod simple;

pub use self::simple::{SimpleGesture, SimpleGestureEngine};

/// Gesture engine.
///
/// A type that tracks a motion and recognizes gestures when the motion is
/// finished.
pub trait GestureEngine {
    /// Recognized gesture.
    type Gesture;

    /// Continues current gesture with next `dataset`.
    fn advance(&mut self, dataset: [u8; 4]);

    /// Finishes current gesture and returns a gesture result if it was
    /// recognized.
    fn finish(&mut self) -> Option<Self::Gesture>;
}
