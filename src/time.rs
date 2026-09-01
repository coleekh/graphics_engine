
#[derive(Debug, Clone, Copy)]
pub struct Clock {
    /// The time at which the current frame started.
    frame_time: std::time::Instant,
    /// Time elapsed since last frame, in seconds.
    delta_time_secs: f64,
    /// Total time elapsed since the start of the program, in seconds.
    total_time_secs: f64,
}

impl Clock {
    pub fn now() -> Self {
        let now = std::time::Instant::now();
        Self {
            frame_time: now,
            delta_time_secs: 0.0,
            total_time_secs: 0.0,
        }
    }

    /// Update the clock for the next frame. Returns old [`Clock`].
    pub fn tick(&mut self) -> Self {
        let prev = *self;
        let now = std::time::Instant::now();
        self.delta_time_secs = (now - self.frame_time).as_secs_f64();
        self.total_time_secs += self.delta_time_secs;
        self.frame_time = now;
        prev
    }
    
    /// Returns the updated clock for the next frame.
    pub fn next_frame(&self) -> Self {
        let frame_time = std::time::Instant::now();
        let delta_time_secs = (frame_time - self.frame_time).as_secs_f64();
        let total_time_secs = self.total_time_secs + delta_time_secs;
        Self {
            frame_time,
            delta_time_secs,
            total_time_secs,
        }
    }

    /// The time at which the current frame started.
    pub fn time(&self) -> std::time::Instant {
        self.frame_time
    }

    /// Get the time elapsed since the last frame, in seconds.
    pub fn delta_time(&self) -> f64 {
        self.delta_time_secs
    }

    /// Get the total time elapsed since the start of the program, in seconds.
    pub fn total_time(&self) -> f64 {
        self.total_time_secs
    }
}