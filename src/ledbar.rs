extern crate sysfs_gpio;

use sysfs_gpio::{Direction, Pin};
use sysfs_gpio::Error::Unsupported;
use std::time::Duration;
use std::thread::sleep;

struct LEDBar {
    gpios: Vec<u64>,
    pins: Vec<Pin>
}

impl LEDBar {
    /// Create a new LEDBar.
    /// The gpios are supposed to be provided in order.
    pub fn new(gpios: Vec<u64>) -> LEDBar {
        let pins: Vec<Pin> = gpios.clone().into_iter().map(|p| Pin::new(p)).collect();
        LEDBar{ gpios, pins }
    }

    /// Create a rippling effect on the LEDBar component.
    /// Turns on 1 LED for ms milliseconds, then off and proceeds to the next one.
    pub fn flow(&self, ms: u64) -> sysfs_gpio::Result<()> {
        let iter = self.pins.clone().into_iter();
        for i in 0..self.pins.len() {
            self.on(i)?;
            sleep(Duration::from_millis(ms));
            self.off(i)?;
        }
        Ok(())
    }

    /// Allows accessing the LEDBar by logical indexes.
    /// The lowest PIN receives index 0, the highest gpios.len() - 1.
    /// The pin will be left exported and on when this function returns.
    fn on(&self, i: usize) -> sysfs_gpio::Result<()> {
        let n = self.pins.len();
        if i > n || i < 0 {
            Err(Unsupported("Index too low/high".to_string()))
        } else {
            let pin = self.pins[i];
            pin.export()?;
            pin.set_direction(Direction::High)?;
            pin.set_value(0)?;
            Ok(())
        }

    }

    /// Allows accessing the LEDBar by logical indexes.
    /// The lowest PIN receives index 0, the highest gpios.len() - 1.
    /// The pin will be unexported.
    fn off(&self, i: usize) -> sysfs_gpio::Result<()> {
        let n = self.pins.len();
        if i > n || i < 0 {
            Err(Unsupported("Index too low/high".to_string()))
        } else {
            let pin = self.pins[i];
            pin.unexport()?;
            Ok(())
        }

    }
}
