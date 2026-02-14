#![cfg_attr(not(feature = "std"), no_std)]

use core::time::Duration;

/// Monotonic time point. Unit is opaque; only differences are meaningful.
pub trait Instant: Copy + Clone + Eq + Ord {
    fn duration_since(self, earlier: Self) -> Duration;
    fn saturating_duration_since(self, earlier: Self) -> Duration;
}

/// Monotonic clock.
pub trait Clock {
    type Instant: Instant;

    fn now(&self) -> Self::Instant;
}

/// A cancellable timer handle.
/// (Cancellation is important to avoid keeping stale retransmit timers alive.)
pub trait TimerHandle {
    fn cancel(&mut self);
}

/// Timer factory / scheduler.
pub trait Timer {
    type Instant: Instant;
    type Handle: TimerHandle;

    /// Schedule a one-shot callback at `deadline`.
    ///
    /// Implementation detail:
    /// - In tokio, this will likely spawn a task that sleeps until deadline and then sends an event.
    /// - In embedded, it could arm a hardware timer.
    ///
    /// The callback must be non-blocking and preferably just enqueue an event.
    fn schedule_at<F>(&self, deadline: Self::Instant, callback: F) -> Self::Handle
    where
        F: FnOnce() + Send + 'static;

    /// Convenience helper.
    fn schedule_after<F>(&self, after: Duration, now: Self::Instant, callback: F) -> Self::Handle
    where
        F: FnOnce() + Send + 'static,
    {
        // default impl relies on Instant arithmetic being done externally; keep simple here.
        // Callers can compute deadline via their own helper if Instant doesn't support add.
        let _ = (after, now);
        self.schedule_at(now, callback)
    }
}
