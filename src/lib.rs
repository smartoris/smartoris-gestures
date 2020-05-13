//! Gestures recognition library for [Drone OS].
//!
//! [Drone OS]: https://www.drone-os.com/
//!
//! # Usage
//!
//! Add the crate to your `Cargo.toml` dependencies:
//!
//! ```toml
//! [dependencies]
//! smartoris-gestures = { version = "0.1.0" }
//! ```
//!
//! Add or extend `std` feature as follows:
//!
//! ```toml
//! [features]
//! std = ["smartoris-gestures/std"]
//! ```

#![feature(never_type)]
#![feature(prelude_import)]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::module_name_repetitions)]

pub mod engines;

mod gestures;

pub use self::{
    engines::GestureEngine,
    gestures::{Gain, Gestures, GesturesSetup, LedDriveCurrent, LedPulseLength},
};

#[prelude_import]
#[allow(unused_imports)]
use drone_core::prelude::*;
