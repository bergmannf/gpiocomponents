extern crate sysfs_gpio;

use sysfs_gpio::Direction;
use sysfs_gpio::Edge;
use sysfs_gpio::Pin;
use sysfs_gpio::Error;
use std::option::Option;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use std::thread::sleep;

const SECOND_IN_NANOS: u64 = 1_000_000_000;
const MILLISECONDS_IN_NANOS: u64 = 1_000_000;
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
        echo.set_edge(Edge::BothEdges)?;
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

    fn await_reading(&self, activation_level: u8) -> Option<u64> {
        let mut start_time = SystemTime::now();
        let timeout_ms = (f64::ceil(self.timeout.to_nanos() as f64 / MILLISECONDS_IN_NANOS as f64)) as isize;
        let mut poller = self.echo.get_poller().unwrap();
        match poller.poll(timeout_ms) {
            Ok(v) => match v {
                Some(p) => if p != activation_level {
                    return None
                }
                None => {
                    return None
                }
            }
            Err(_) => return None,
        };
        start_time = SystemTime::now();
        match poller.poll(timeout_ms) {
            Ok(v) => match v {
                Some(p) => if p == activation_level {
                    return None
                }
                None => {
                    return None
                }
            }
            Err(_) => return None,
        };
        let pulse_time = SystemTime::now();
        match pulse_time.duration_since(start_time) {
            Ok(t) => Some(t.to_nanos()),
            Err(_) => None,
        }
    }

    /// Return the distance in centimeters that was measured by the sensor.
    /// If there was no distance measured the result will be None.
    pub fn pulse(&self) -> f64 {
        self.trigger.set_value(1);
        sleep(Duration::new(0, 10000));
        self.trigger.set_value(0);
        let ping_time = self.await_reading(1);
        match ping_time {
            Some(t) => (t as f64 / SECOND_IN_NANOS as f64) * SPEED_OF_SOUND_CM as f64 / 2.0,
            None => -1.0,
        }
    }
}
