pub struct Rate {
    ongoing_sample: f64,
    available_sample: Option<f64>,
    ongoing_sample_time_used: f64,
    bucket_size: f64
}

impl Rate {
    pub fn new(bucket_size: f64) -> Rate {
        Rate {
            ongoing_sample: 0.,
            available_sample: None,
            ongoing_sample_time_used: 0.,
            bucket_size        
        }
    }

    pub fn add_sample(&mut self, interval: f64, value: f64) {
        let rate = value / interval;
        let time_remaining = self.bucket_size - self.ongoing_sample_time_used;
        if interval < time_remaining {
            /* not enough to finish a sample */
            self.ongoing_sample += value;
            self.ongoing_sample_time_used += interval;
        } else if interval < time_remaining + self.bucket_size {
            /* enough to finish a smaple and start a new one */
            self.available_sample = Some(self.ongoing_sample + rate * time_remaining);
            self.ongoing_sample = rate * (interval-time_remaining);
            self.ongoing_sample_time_used = interval-time_remaining;
        } else {
            /* enough to finish a sample and fill at least one */
            self.available_sample = Some(rate * self.bucket_size);
            let tail_interval = (interval - time_remaining) % self.bucket_size;
            self.ongoing_sample = rate * tail_interval;
            self.ongoing_sample_time_used = tail_interval;
        }
    }

    pub fn sample(&self) -> Option<f64> { self.available_sample }
}
