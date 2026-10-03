//! Managed timing outside the unchanged official-client simulation rules.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const CAP: Duration = Duration::from_secs(12 * 60 * 60);
pub const CHECKPOINT_INTERVAL: Duration = Duration::from_secs(1);
const BROWSER_TICK_NS: u64 = 100_000_000;

#[derive(Debug, Error)]
pub enum TimingError {
    #[error("could not read runtime clock: {0}")]
    Clock(#[from] std::io::Error),
    #[error("runtime clock value is out of range")]
    ClockRange,
    #[error("rested runtime metadata is invalid")]
    InvalidMetadata,
    #[error("runtime monotonic clock moved backwards")]
    BackwardsMonotonic,
    #[error("runtime duration is too large")]
    DurationOverflow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ClockSample {
    pub wall_ms: i64,
    pub awake: Duration,
    pub suspend: Duration,
}

impl ClockSample {
    pub fn now() -> Result<Self, TimingError> {
        let boot = clock(libc::CLOCK_BOOTTIME)?;
        let awake = clock(libc::CLOCK_MONOTONIC)?;
        Ok(Self {
            wall_ms: wall_millis()?,
            awake,
            suspend: boot.saturating_sub(awake),
        })
    }
}

fn clock(id: libc::clockid_t) -> Result<Duration, TimingError> {
    let mut value = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    // The pointer references an initialized, writable timespec for this call.
    if unsafe { libc::clock_gettime(id, &mut value) } != 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    let seconds = u64::try_from(value.tv_sec).map_err(|_| TimingError::ClockRange)?;
    let nanos = u32::try_from(value.tv_nsec).map_err(|_| TimingError::ClockRange)?;
    if nanos >= 1_000_000_000 {
        return Err(TimingError::ClockRange);
    }
    Ok(Duration::new(seconds, nanos))
}

pub(crate) fn wall_millis() -> Result<i64, TimingError> {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| TimingError::ClockRange)?;
    i64::try_from(elapsed.as_millis()).map_err(|_| TimingError::ClockRange)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RestedState {
    balance_ns: u64,
    browser_remainder_ns: u64,
    pub accounted_wall_ms: i64,
    pub active: bool,
}

impl RestedState {
    pub fn empty(wall_ms: i64) -> Self {
        Self {
            balance_ns: 0,
            browser_remainder_ns: 0,
            accounted_wall_ms: wall_ms,
            active: false,
        }
    }

    pub fn validate(self) -> Result<Self, TimingError> {
        if self.balance_ns > CAP.as_nanos() as u64
            || self.browser_remainder_ns >= BROWSER_TICK_NS
            || self.accounted_wall_ms < 0
        {
            return Err(TimingError::InvalidMetadata);
        }
        Ok(self)
    }

    pub fn balance(self) -> Duration {
        Duration::from_nanos(self.balance_ns)
    }

    pub fn accrue(&mut self, elapsed: Duration) {
        self.balance_ns =
            (u128::from(self.balance_ns) + elapsed.as_nanos()).min(CAP.as_nanos()) as u64;
    }

    pub fn resume(&mut self, wall_ms: i64) {
        self.accrue(Duration::from_millis(
            wall_ms.saturating_sub(self.accounted_wall_ms).max(0) as u64,
        ));
        self.record(wall_ms, true);
    }

    pub fn record(&mut self, wall_ms: i64, active: bool) {
        self.accounted_wall_ms = self.accounted_wall_ms.max(wall_ms);
        self.active = active;
    }

    /// Spend the entire awake interval, but only credit its bounded contribution.
    pub fn spend(
        &mut self,
        awake: Duration,
        contributing: Duration,
    ) -> Result<Duration, TimingError> {
        let bonus = self.balance().min(contributing.min(awake));
        let earned = contributing
            .min(awake)
            .checked_add(bonus)
            .ok_or(TimingError::DurationOverflow)?;
        self.balance_ns -= self.balance().min(awake).as_nanos() as u64;
        Ok(earned)
    }

    pub fn browser_ticks(&mut self, virtual_time: Duration) -> Result<u64, TimingError> {
        let total = virtual_time.as_nanos() + u128::from(self.browser_remainder_ns);
        let ticks = total / u128::from(BROWSER_TICK_NS);
        let millis = u64::try_from(
            ticks
                .checked_mul(100)
                .ok_or(TimingError::DurationOverflow)?,
        )
        .map_err(|_| TimingError::DurationOverflow)?;
        self.browser_remainder_ns = (total % u128::from(BROWSER_TICK_NS)) as u64;
        Ok(millis)
    }

    pub fn browser_remainder(self) -> Duration {
        Duration::from_nanos(self.browser_remainder_ns)
    }

    pub fn view(self, wall_ms: i64, owned: bool) -> RestedView {
        let mut projected = self;
        let elapsed =
            Duration::from_millis(wall_ms.saturating_sub(self.accounted_wall_ms).max(0) as u64);
        if owned && self.active {
            projected.balance_ns -= projected.balance().min(elapsed).as_nanos() as u64;
        } else if !owned {
            projected.accrue(elapsed);
        }
        RestedView {
            available_ms: projected.balance().as_millis() as u64,
            active_multiplier: if owned && projected.balance_ns > 0 {
                2
            } else {
                1
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestedView {
    pub available_ms: u64,
    pub active_multiplier: u8,
}

impl Default for RestedView {
    fn default() -> Self {
        Self {
            available_ms: 0,
            active_multiplier: 1,
        }
    }
}

/// Real time needed to earn the supplied virtual time, splitting at exhaustion.
pub(crate) fn real_duration(virtual_time: Duration, balance: Duration) -> Duration {
    let boosted_virtual = virtual_time.min(balance.saturating_mul(2));
    boosted_virtual.div_f64(2.0) + virtual_time.saturating_sub(boosted_virtual)
}

pub(crate) fn observe(
    previous: ClockSample,
    current: ClockSample,
) -> Result<(Duration, Duration), TimingError> {
    let awake = current
        .awake
        .checked_sub(previous.awake)
        .ok_or(TimingError::BackwardsMonotonic)?;
    let sleep = current.suspend.saturating_sub(previous.suspend);
    // Sequential clock reads can differ by microseconds; do not bank that jitter.
    Ok((
        awake,
        if sleep >= Duration::from_millis(1) {
            sleep
        } else {
            Duration::ZERO
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(awake: u64, sleep: u64) -> ClockSample {
        ClockSample {
            wall_ms: 1_000,
            awake: Duration::from_secs(awake),
            suspend: Duration::from_secs(sleep),
        }
    }

    #[test]
    fn caps_rest_preserves_interruptions_and_ignores_backwards_wall_time() {
        let mut state = RestedState::empty(1_000);
        state.resume(8 * 3_600_000 + 1_000);
        assert_eq!(state.balance(), Duration::from_secs(8 * 3_600));
        state
            .spend(Duration::from_secs(3 * 3_600), Duration::ZERO)
            .unwrap();
        state.record(11 * 3_600_000 + 1_000, false);
        state.resume(15 * 3_600_000 + 1_000);
        assert_eq!(state.balance(), Duration::from_secs(9 * 3_600));
        state.resume(1_000);
        assert_eq!(state.balance(), Duration::from_secs(9 * 3_600));
        state.resume(15 * 3_600_000 + 1_000);
        assert_eq!(state.balance(), Duration::from_secs(9 * 3_600));
        state.accrue(Duration::from_secs(20 * 3_600));
        assert_eq!(state.balance(), CAP);
    }

    #[test]
    fn exhaustion_buffers_fractional_ticks_and_spends_awake_delays() {
        let mut state = RestedState::empty(0);
        state.accrue(Duration::from_millis(250));
        let earned = state
            .spend(Duration::from_secs(1), Duration::from_secs(1))
            .unwrap();
        assert_eq!(earned, Duration::from_millis(1_250));
        assert_eq!(state.browser_ticks(earned).unwrap(), 1_200);
        assert_eq!(state.browser_remainder(), Duration::from_millis(50));
        assert_eq!(state.browser_ticks(Duration::from_millis(50)).unwrap(), 100);
        state.accrue(Duration::from_secs(20));
        assert_eq!(
            state
                .spend(Duration::from_secs(10), Duration::ZERO)
                .unwrap(),
            Duration::ZERO
        );
        assert_eq!(state.balance(), Duration::from_secs(10));
    }

    #[test]
    fn desktop_fractional_periods_do_not_lose_spending_precision() {
        let mut state = RestedState::empty(0);
        state.accrue(Duration::from_secs(10));
        let period = Duration::from_nanos(54_687_500);
        assert_eq!(
            real_duration(Duration::from_nanos(109_375_000), state.balance()),
            period
        );
        for _ in 0..128 {
            assert_eq!(state.spend(period, period).unwrap(), period * 2);
        }
        assert_eq!(state.balance(), Duration::from_secs(3));
        assert_eq!(
            real_duration(Duration::from_millis(100), Duration::from_millis(25)),
            Duration::from_millis(75)
        );
    }

    #[test]
    fn sleep_is_separate_from_awake_stalls_and_is_accounted_once() {
        assert_eq!(
            observe(sample(1, 0), sample(11, 0)).unwrap(),
            (Duration::from_secs(10), Duration::ZERO)
        );
        assert_eq!(
            observe(sample(11, 0), sample(12, 14_400)).unwrap(),
            (Duration::from_secs(1), Duration::from_secs(14_400))
        );
        assert_eq!(
            observe(sample(12, 14_400), sample(13, 14_400)).unwrap(),
            (Duration::from_secs(1), Duration::ZERO)
        );
        assert!(matches!(
            observe(sample(2, 0), sample(1, 0)),
            Err(TimingError::BackwardsMonotonic)
        ));
    }

    #[test]
    fn invalid_metadata_clock_errors_and_read_only_projection_are_explicit() {
        assert!(clock(-1).is_err());
        ClockSample::now().unwrap();
        let mut state = RestedState::empty(1_000);
        assert_eq!(
            state.view(3_601_000, false),
            RestedView {
                available_ms: 3_600_000,
                active_multiplier: 1
            }
        );
        assert_eq!(state.balance(), Duration::ZERO);
        state.balance_ns = CAP.as_nanos() as u64 + 1;
        assert!(state.validate().is_err());
    }
}
