use crate::engines::GestureEngine;
use core::{convert::TryInto, marker::PhantomData};
use smartoris_apds9960::{Apds9960Drv, Apds9960I2CPort};

/// Gestures setup.
pub struct GesturesSetup<E: GestureEngine> {
    /// A gesture engine. See [`engines`](crate::engines) for available options.
    pub engine: E,
    /// Offset compensation for UP photo-diode in gesture mode.
    pub goffset_up: u8,
    /// Offset compensation for DOWN photo-diode in gesture mode.
    pub goffset_down: u8,
    /// Offset compensation for LEFT photo-diode in gesture mode.
    pub goffset_left: u8,
    /// Offset compensation for RIGHT photo-diode in gesture mode.
    pub goffset_right: u8,
    /// Offset compensation for UP and RIGHT photo-diodes in proximity mode.
    pub poffset_ur: u8,
    /// Offset compensation for DOWN and LEFT photo-diodes in proximity mode.
    pub poffset_dl: u8,
    /// LED drive current.
    pub led_drive: LedDriveCurrent,
    /// LED pulse length.
    pub led_pulse: LedPulseLength,
    /// Gain control,
    pub gain: Gain,
    /// Gesture entry threshold.
    pub entry_threshold: u8,
    /// Gesture exit threshold.
    pub exit_threshold: u8,
}

/// Gestures driver.
pub struct Gestures<A, E: GestureEngine> {
    adapters: PhantomData<A>,
    engine: E,
    exit_threshold: u8,
}

/// LED drive current.
#[derive(Clone, Copy)]
pub enum LedDriveCurrent {
    /// LED drive current - 12.5 mA, LED boost - 100%
    D12_5,
    /// LED drive current - 18.75 mA, LED boost - 150%
    D18_75,
    /// LED drive current - 25 mA, LED boost - 100%
    D25,
    /// LED drive current - 37.5 mA, LED boost - 150%
    D37_5,
    /// LED drive current - 50 mA, LED boost - 100%
    D50,
    /// LED drive current - 75 mA, LED boost - 150%
    D75,
    /// LED drive current - 100 mA, LED boost - 100%
    D100,
    /// LED drive current - 150 mA, LED boost - 150%
    D150,
    /// LED drive current - 200 mA, LED boost - 200%
    D200,
    /// LED drive current - 300 mA, LED boost - 300%
    D300,
}

/// LED pulse length.
#[derive(Clone, Copy)]
pub enum LedPulseLength {
    /// 4 µs
    P4 = 0,
    /// 8 µs
    P8 = 1,
    /// 16 µs
    P16 = 2,
    /// 32 µs
    P32 = 3,
}

/// Gain.
#[derive(Clone, Copy)]
pub enum Gain {
    /// 1×
    X1 = 0,
    /// 2×
    X2 = 1,
    /// 4×
    X4 = 2,
    /// 8×
    X8 = 3,
}

impl<A, E: GestureEngine> Gestures<A, E> {
    /// Sets up a new [`Gestures`] from `setup` values.
    ///
    /// # Errors
    ///
    /// If `i2c` implementation returns `Err`, it's propagated to the caller.
    pub async fn init<P: Apds9960I2CPort<A>>(
        apds9960: &mut Apds9960Drv<A>,
        i2c: &mut P,
        setup: GesturesSetup<E>,
    ) -> Result<Self, P::Error> {
        let GesturesSetup {
            engine,
            goffset_up,
            goffset_down,
            goffset_left,
            goffset_right,
            poffset_ur,
            poffset_dl,
            led_drive,
            led_pulse,
            gain,
            entry_threshold,
            exit_threshold,
        } = setup;
        apds9960
            .store_enable(i2c, |r| {
                r.set_pon() // power on
                    .set_pen() // proximity detect enable
                    .set_gen() // gesture enable
            })
            .await?;
        apds9960
            .store_ppulse(i2c, |r| {
                r.write_pplen(led_pulse as u8) // proximity pulse length
                    .write_ppulse(8) // proximity pulse count (recommended by the manufacturer)
            })
            .await?;
        apds9960
            .store_control(i2c, |r| {
                r.write_pgain(gain as u8) // proximity gain control
                    .write_ldrive(led_drive.drive()) // LED drive strength
            })
            .await?;
        apds9960
            .store_config2(i2c, |r| {
                r.write_led_boost(led_drive.boost()) // additional LED current
            })
            .await?;
        apds9960.store_poffset_ur(i2c, poffset_ur).await?; // proximity offset for up and right
        apds9960.store_poffset_dl(i2c, poffset_dl).await?; // proximity offset for down and left
        apds9960.store_gpenth(i2c, entry_threshold).await?; // gesture proximity entry threshold
        apds9960
            .store_gconf1(i2c, |r| {
                r.write_gfifoth(2) // gesture FIFO threshold - 8 datasets
            })
            .await?;
        apds9960
            .store_gconf2(i2c, |r| {
                r.write_ggain(gain as u8) // gesture gain control
                    .write_gldrive(led_drive.drive()) // gesture LED drive strength
                    .write_gwtime(0) // gesture wait time
            })
            .await?;
        apds9960.store_goffset_u(i2c, goffset_up).await?; // gesture up offset
        apds9960.store_goffset_d(i2c, goffset_down).await?; // gesture down offset
        apds9960.store_goffset_l(i2c, goffset_left).await?; // gesture left offset
        apds9960.store_goffset_r(i2c, goffset_right).await?; // gesture right offset
        apds9960
            .store_gpulse(i2c, |r| {
                r.write_gplen(led_pulse as u8) // gesture pulse length
                    .write_gpulse(8) // number of gesture pulses (recommended by the manufacturer)
            })
            .await?;
        apds9960
            .store_gconf4(i2c, |r| {
                r.set_gfifo_clr() // clears GFIFO, GIN, GVALID, GFIFO_OV and GFIFO_LVL
                    .set_gien() // gesture interrupt enable
                    .clear_gmode() // gesture mode
            })
            .await?;
        Ok(Self { adapters: PhantomData, engine, exit_threshold })
    }

    /// Reads a next portion of sensor data, and returns a gesture result if it
    /// was recognized.
    ///
    /// # Errors
    ///
    /// If `i2c` implementation returns `Err`, it's propagated to the caller.
    pub async fn advance<P: Apds9960I2CPort<A>>(
        &mut self,
        apds9960: &mut Apds9960Drv<A>,
        i2c: &mut P,
    ) -> Result<Option<E::Gesture>, P::Error> {
        let gflvl = apds9960.load_gflvl(i2c).await?;
        if gflvl > 0 {
            let datasets = apds9960.drain_fifo(i2c, gflvl).await?;
            // log::Port::new(2).write_bytes(datasets);
            for i in (0..datasets.len()).step_by(4) {
                let dataset: [u8; 4] = datasets[i..i + 4].try_into().unwrap();
                if dataset.iter().any(|&x| x > self.exit_threshold) {
                    self.engine.advance(dataset);
                } else {
                    apds9960
                        .store_gconf4(i2c, |r| r.set_gfifo_clr().set_gien().clear_gmode())
                        .await?;
                    return Ok(self.engine.finish());
                }
            }
        }
        Ok(None)
    }
}

impl LedDriveCurrent {
    fn drive(self) -> u8 {
        match self {
            Self::D100 | Self::D150 | Self::D200 | Self::D300 => 0,
            Self::D50 | Self::D75 => 1,
            Self::D25 | Self::D37_5 => 2,
            Self::D12_5 | Self::D18_75 => 3,
        }
    }

    fn boost(self) -> u8 {
        match self {
            Self::D100 | Self::D50 | Self::D25 | Self::D12_5 => 0,
            Self::D150 | Self::D75 | Self::D37_5 | Self::D18_75 => 1,
            Self::D200 => 2,
            Self::D300 => 3,
        }
    }
}
