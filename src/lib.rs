//! Hand gestures sensing library for [Drone OS].
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

#![feature(prelude_import)]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![cfg_attr(not(feature = "std"), no_std)]

#[prelude_import]
#[allow(unused_imports)]
use drone_core::prelude::*;
