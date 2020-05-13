use crate::engines::GestureEngine;
use core::cmp::Ordering;
use libm::fabsf;

/// A gesture engine that tracks simple upward, downward, leftward, rightward
/// gestures.
pub struct SimpleGestureEngine {
    active: bool,
    prev_x: f32,
    prev_y: f32,
    move_x: f32,
    move_y: f32,
}

/// A gesture recognized by [`SimpleGestureEngine`].
#[derive(Debug)]
pub enum SimpleGesture {
    /// Upward motion.
    Up,
    /// Downward motion.
    Down,
    /// Leftward motion.
    Left,
    /// Rightward motion.
    Right,
}

impl GestureEngine for SimpleGestureEngine {
    type Gesture = SimpleGesture;

    fn advance(&mut self, dataset: [u8; 4]) {
        let [up, down, left, right] = dataset;
        let y = position(up, down);
        let x = position(left, right);
        #[cfg(feature = "log-gesture-positions")]
        log_gesture_positions(x, y);
        if self.active {
            self.move_x += self.prev_x - x;
            self.move_y += self.prev_y - y;
        } else {
            self.active = true;
            self.move_x = 0.0;
            self.move_y = 0.0;
        }
        self.prev_x = x;
        self.prev_y = y;
    }

    fn finish(&mut self) -> Option<SimpleGesture> {
        self.active = false;
        if fabsf(self.move_y) > fabsf(self.move_x) {
            if self.move_y > 0.0 { Some(SimpleGesture::Up) } else { Some(SimpleGesture::Down) }
        } else if fabsf(self.move_x) > fabsf(self.move_y) {
            if self.move_x > 0.0 { Some(SimpleGesture::Left) } else { Some(SimpleGesture::Right) }
        } else {
            None
        }
    }
}

impl Default for SimpleGestureEngine {
    fn default() -> Self {
        Self { active: false, prev_x: 0.0, prev_y: 0.0, move_x: 0.0, move_y: 0.0 }
    }
}

fn position(a: u8, b: u8) -> f32 {
    let pos = i16::from(a) + i16::from(b) - i16::from(u8::MAX) * 2;
    match a.cmp(&b) {
        Ordering::Greater => f32::from(pos) * (1.0 - f32::from(b) / f32::from(a)),
        Ordering::Less => f32::from(pos) * (f32::from(a) / f32::from(b) - 1.0),
        Ordering::Equal => 0.0,
    }
}

#[cfg(feature = "log-gesture-positions")]
fn log_gesture_positions(x: f32, y: f32) {
    drone_core::log::Port::new(3).write(x.to_bits()).write(y.to_bits());
}
