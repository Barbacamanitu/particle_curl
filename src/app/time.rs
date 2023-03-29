use std::time::{Duration, Instant};

pub struct Time {
    last_frame: Instant,
    last_fps_check: Instant,
    fps_update_duration: Duration,
    frame_since_last_fps_check: u32,
    start_time: Instant,
    render_ticks: usize,
}

pub struct FPSData {
    pub render_fps: f32,
}

impl Time {
    pub fn new(fps_update_duration: Duration) -> Time {
        Time {
            last_frame: Instant::now(),
            last_fps_check: Instant::now(),
            fps_update_duration: fps_update_duration,
            frame_since_last_fps_check: 0,
            start_time: Instant::now(),
            render_ticks: 0,
        }
    }

    pub fn get_elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    //Returns None if not enough time has passed since the last FPS check. *
    pub fn get_fps(&mut self) -> Option<FPSData> {
        let elapsed = self.last_fps_check.elapsed();
        if elapsed > self.fps_update_duration {
            let fps =
                (self.frame_since_last_fps_check as f32) / self.fps_update_duration.as_secs_f32();
            self.frame_since_last_fps_check = 0;
            self.last_fps_check = Instant::now();

            return Some(FPSData { render_fps: fps });
        }
        None
    }

    pub fn render_tick(&mut self) {
        self.last_frame = Instant::now();
        self.frame_since_last_fps_check += 1;
        self.render_ticks += 1;
    }

    pub fn render_ticks(&self) -> usize {
        self.render_ticks
    }
}
