extern crate sysfs_gpio;

use sysfs_gpio::Direction;
use sysfs_gpio::Pin;
use sysfs_gpio::Error;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use std::thread::sleep;

const SECOND_IN_NANOS: u64 = 1_000_000_000;
const SPEED_OF_SOUND: u64 = 340; // (m / s)
const SPEED_OF_SOUND_CM: u64 = SPEED_OF_SOUND * 100; // (cm / s)

trait Nanoseconds {
    fn to_nanos(&self) -> u64;
}

impl Nanoseconds for Duration {
    fn to_nanos(&self) -> u64 {
        let secs = self.as_secs();
        let nanos = self.subsec_nanos() as u64;
        let secs_nano = secs * SECOND_IN_NANOS;
        secs_nano + nanos
    }
}

pub struct Sonar {
    echo: Pin,
    trigger: Pin,
    timeout: Duration,
    max_distance: u64,
}

/// Implementation for a HC SR501 ultrasonic ranging module.
/// The sonar will only produce a reading when the `pulse` method is called.
/// This can stall the calling thread while waiting for an answer.
impl Sonar {
    pub fn new(echo: Pin, trigger: Pin, max_distance: u64) -> Result<Sonar, Error> {
        let total_timeout = f64::floor(
            max_distance as f64 / SPEED_OF_SOUND_CM as f64 *
                SECOND_IN_NANOS as f64 * 2.0,
        );
        let timeout_secs = f64::floor(total_timeout / SECOND_IN_NANOS as f64);
        let timeout_nanos = f64::floor(total_timeout - (timeout_secs * SECOND_IN_NANOS as f64));
        let timeout = Duration::new(timeout_secs as u64, timeout_nanos as u32);
        if !echo.is_exported() {
            echo.export()?;
        };
        echo.set_direction(Direction::In)?;
        if !trigger.is_exported() {
            trigger.export()?;
        }
        trigger.set_direction(Direction::Out)?;
        Ok(Sonar {
            echo: echo,
            trigger: trigger,
            timeout: timeout,
            max_distance: max_distance,
        })
    }

    fn await_reading(&self, activation_level: u8) -> u64 {
        let mut start_time = SystemTime::now();
        while {
            match self.echo.get_value() {
                Ok(v) => v != activation_level,
                Err(_) => false,
            }
        }
        {
            if {
                match SystemTime::now().duration_since(start_time) {
                    Ok(t) => t > self.timeout,
                    Err(_) => false,
                }
            }
            {
                return 0;
            }
        }
        start_time = SystemTime::now();
        while {
            match self.echo.get_value() {
                Ok(v) => v == activation_level,
                Err(_) => false,
            }
        }
        {
            if {
                match SystemTime::now().duration_since(start_time) {
                    Ok(t) => t > self.timeout,
                    Err(_) => false,
                }
            }
            {
                return 0;
            }
        }
        let pulse_time = SystemTime::now();
        match pulse_time.duration_since(start_time) {
            Ok(t) => t.to_nanos(),
            Err(_) => 0,
        }
    }

    /// Return the distance in centimeters that was measured by the sensor.
    /// If there was no distance measured the result will be None.
    pub fn pulse(&self) -> f64 {
        self.trigger.set_value(1);
        sleep(Duration::new(0, 10000));
        self.trigger.set_value(0);
        let ping_time = self.await_reading(1);
        let distance = (ping_time as f64 / SECOND_IN_NANOS as f64) * SPEED_OF_SOUND_CM as f64 / 2.0;
        distance
    }
}
