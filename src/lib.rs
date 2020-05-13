//! Gestures recognition library for [Drone OS]. Based on APDS-9960 digital
//! proximity, ambient light, RGB and gesture sensor.
//!
//! [Drone OS]: https://www.drone-os.com/
//!
//! # Usage
//!
//! Add the crate to your `Cargo.toml` dependencies:
//!
//! ```toml
//! [dependencies]
//! smartoris-apds9960 = { version = "0.1.0" }
//! smartoris-gestures = { version = "0.1.0" }
//! ```
//!
//! Add or extend `std` feature as follows:
//!
//! ```toml
//! [features]
//! std = ["smartoris-apds9960/std", "smartoris-gestures/std"]
//! ```
//!
//! The library can be used with any EXTI implementation. Here is an example of
//! integration with [`smartoris-exti`](https://crates.io/crates/smartoris-exti)
//! crate.
//!
//! ```no_run
//! # #![feature(const_fn)]
//! # #![feature(never_type)]
//! # #![feature(unwrap_infallible)]
//! # use drone_core::token::Token;
//! # use drone_cortexm::thr::prelude::*;
//! # use drone_stm32_map::stm32_reg_tokens;
//! # stm32_reg_tokens! {
//! #     struct Regs;
//! #     !scb_ccr;
//! #     !mpu_type; !mpu_ctrl; !mpu_rnr; !mpu_rbar; !mpu_rasr;
//! # }
//! # mod thr {
//! #     use drone_stm32_map::thr::*;
//! #     drone_cortexm::thr::vtable! {
//! #         use Thr;
//! #         pub struct Vtable;
//! #         pub struct Handlers;
//! #         pub struct Thrs;
//! #         pub struct ThrsInit;
//! #         static THREADS;
//! #         pub 10: EXTI4;
//! #         pub 16: DMA1_CH5;
//! #         pub 17: DMA1_CH6;
//! #         pub 31: I2C1_EV;
//! #         pub 32: I2C1_ER;
//! #     }
//! #     drone_cortexm::thr! {
//! #         use THREADS;
//! #         pub struct Thr {}
//! #         pub struct ThrLocal {}
//! #     }
//! # }
//! # struct Adapters;
//! # #[async_trait::async_trait]
//! # impl<
//! #     I2C: I2CMap,
//! #     I2CEv: IntToken,
//! #     I2CEr: IntToken,
//! #     DmaTx: DmaChMap,
//! #     DmaTxInt: IntToken,
//! #     DmaRx: DmaChMap,
//! #     DmaRxInt: IntToken,
//! # > smartoris_apds9960::Apds9960I2CPort<Adapters>
//! #     for smartoris_i2c::I2CDrv<I2C, I2CEv, I2CEr, DmaTx, DmaTxInt, DmaRx, DmaRxInt>
//! # {
//! #     type Error = !;
//! #     async fn write(
//! #         &mut self,
//! #         addr: u8,
//! #         buf: Box<[u8]>,
//! #         count: usize,
//! #     ) -> Result<Box<[u8]>, (Box<[u8]>, !)> {
//! #         unimplemented!()
//! #     }
//! #     async fn read(
//! #         &mut self,
//! #         addr: u8,
//! #         buf: Box<[u8]>,
//! #         count: usize,
//! #     ) -> Result<Box<[u8]>, (Box<[u8]>, !)> {
//! #         unimplemented!()
//! #     }
//! # }
//! use crate::thr::Thrs;
//! use drone_cortexm::reg::prelude::*;
//! use drone_stm32_map::periph::{
//!     dma::ch::{Dma1Ch5, Dma1Ch6, DmaChMap},
//!     exti::{periph_exti4, Exti4},
//!     gpio::pin::{GpioB4, GpioC13, GpioPinPeriph},
//!     i2c::{I2CMap, I2C1},
//! };
//! use futures::prelude::*;
//! use smartoris_apds9960::Apds9960Drv;
//! use smartoris_exti::{ExtiDrv, ExtiSetup};
//! use smartoris_gestures::{
//!     engines::{SimpleGesture, SimpleGestureEngine},
//!     Gain, Gestures, GesturesSetup, LedDriveCurrent, LedPulseLength,
//! };
//! use smartoris_i2c::I2CDrv;
//!
//! // Suppose APDS-9960 I²C pins connected to I2C1 peripheral, APDS-9960 INT
//! // pin connected to B4 pin, and some LED connected to C13 pin.
//!
//! async fn handler(
//!     reg: Regs,
//!     thr: Thrs,
//!     mut i2c1: I2CDrv<
//!         I2C1,
//!         thr::I2C1Ev,
//!         thr::I2C1Er,
//!         Dma1Ch6,
//!         thr::Dma1Ch6,
//!         Dma1Ch5,
//!         thr::Dma1Ch5,
//!     >,
//!     gpio_b4: drone_stm32_map::periph::gpio::pin::GpioPinPeriph<GpioB4>,
//!     gpio_c13: drone_stm32_map::periph::gpio::pin::GpioPinPeriph<GpioC13>,
//! ) {
//!     // APDS-9960 interrupt events stream read from B4 pin.
//!     let exti4 = ExtiDrv::init(ExtiSetup {
//!         exti: periph_exti4!(reg),
//!         exti_int: thr.exti4,
//!         config: 0b0001, // PB4 pin
//!         falling: true,  // trigger the interrupt on a falling edge
//!         rising: false,  // don't trigger the interrupt on a rising edge
//!     });
//!
//!     let mut exti4_stream = exti4.create_saturating_stream();
//!     let mut apds9960 = Apds9960Drv::init();
//!     let mut gestures = Gestures::init(&mut apds9960, &mut i2c1, GesturesSetup {
//!         engine: SimpleGestureEngine::default(),
//!         goffset_up: 0x4,
//!         goffset_down: 0,
//!         goffset_left: 0,
//!         goffset_right: 0x4,
//!         poffset_ur: 0,
//!         poffset_dl: 0,
//!         led_drive: LedDriveCurrent::D300, // 300 mA
//!         led_pulse: LedPulseLength::P32,   // 32 µs
//!         gain: Gain::X1,                   // 1×
//!         entry_threshold: 0x1A,
//!         exit_threshold: 0x1A,
//!     })
//!     .await
//!     .into_ok();
//!     // Wait for a falling edge trigger on PB4.
//!     while exti4_stream.next().await.is_some() {
//!         // Repeat until PB4 is back to high level.
//!         while !gpio_b4.gpio_idr_idr.read_bit() {
//!             // Read APDS-9960 FIFO buffer.
//!             match gestures.advance(&mut apds9960, &mut i2c1).await.into_ok() {
//!                 // Turn on some LED.
//!                 Some(SimpleGesture::Up) => gpio_c13.gpio_bsrr_br.set_bit(),
//!                 // Turn off some LED.
//!                 Some(SimpleGesture::Down) => gpio_c13.gpio_bsrr_bs.set_bit(),
//!                 _ => {}
//!             }
//!         }
//!     }
//! }
//! # fn main() {
//! #     unsafe {
//! #         handler(
//! #             Regs::take(),
//! #             drone_cortexm::thr::init(thr::ThrsInit::take()),
//! #             core::mem::MaybeUninit::uninit().assume_init(),
//! #             core::mem::MaybeUninit::uninit().assume_init(),
//! #             core::mem::MaybeUninit::uninit().assume_init(),
//! #         );
//! #     }
//! # }
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
