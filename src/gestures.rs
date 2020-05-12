use crate::engines::GestureEngine;
use core::{convert::TryInto, marker::PhantomData};
use smartoris_apds9960::{Apds9960Drv, Apds9960I2CPort};

/// Gestures setup.
pub struct GesturesSetup<E: GestureEngine> {
    /// A gesture engine. See [`engines`](crate::engines) for available options.
    pub engine: E,
    /// Offset compensation for UP photo-diode.
    pub offset_up: u8,
    /// Offset compensation for DOWN photo-diode.
    pub offset_down: u8,
    /// Offset compensation for LEFT photo-diode.
    pub offset_left: u8,
    /// Offset compensation for RIGHT photo-diode.
    pub offset_right: u8,
    /// Gesture entry threshold.
    pub entry_threshold: u8,
}

/// Gestures driver.
pub struct Gestures<A, E: GestureEngine> {
    adapters: PhantomData<A>,
    engine: E,
    entry_threshold: u8,
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
            offset_up,
            offset_down,
            offset_left,
            offset_right,
            entry_threshold,
        } = setup;
        apds9960.store_enable(i2c, |r| r.set_pon().set_pen().set_gen()).await?;
        apds9960.store_gpenth(i2c, entry_threshold).await?;
        apds9960.store_goffset_u(i2c, offset_up).await?;
        apds9960.store_goffset_d(i2c, offset_down).await?;
        apds9960.store_goffset_l(i2c, offset_left).await?;
        apds9960.store_goffset_r(i2c, offset_right).await?;
        apds9960.store_ppulse(i2c, |r| r.write_pplen(3).write_ppulse(4)).await?;
        apds9960.store_gpulse(i2c, |r| r.write_gplen(3).write_gpulse(4)).await?;
        apds9960.store_control(i2c, |r| r.write_ldrive(0)).await?;
        apds9960.store_config2(i2c, |r| r.write_led_boost(3)).await?;
        apds9960
            .store_gconf1(i2c, |r| r.write_gfifoth(2).write_gexmsk(0b0000).write_gexpers(0))
            .await?;
        apds9960.store_gconf2(i2c, |r| r.write_ggain(0).write_gldrive(0).write_gwtime(0)).await?;
        apds9960.store_gconf4(i2c, |r| r.set_gfifo_clr().set_gien().clear_gmode()).await?;
        Ok(Self { adapters: PhantomData, engine, entry_threshold })
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
                if dataset.iter().any(|&x| x > self.entry_threshold) {
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
