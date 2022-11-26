const MAX_TRIGGERS: usize = 128;


#[derive(Debug, Default, Clone, Copy)]
pub struct Trigger {
    location: u64,
    destination_channel_index: usize,
    destination_location: u64
}

pub struct ControlLoop {
    triggers: [Trigger; MAX_TRIGGERS],
    length: u64,
    location: u64,
    playing: bool,
    samples_per_interval: u64
}

impl ControlLoop {
    pub fn new(length: u64) -> ControlLoop {
        Self { 
            triggers: [Trigger::default(); MAX_TRIGGERS],
            length,
            location: 0,
            playing: false,
            samples_per_interval: 24_000
        }
    }

    pub fn play(&mut self) {
        self.playing = true;
    }

    pub fn stop(&mut self) {
        self.playing = false;
    }

    fn update_on_interval(&mut self) {

    }

    pub fn update(&mut self) {
        if self.playing {
            if self.location % self.samples_per_interval == 0 {
                self.update_on_interval();
            }
            self.location = (self.location + 1) % (self.length * self.samples_per_interval);
        }
    }
}

pub fn bpm_to_spb(bpm: u32, sample_rate: u32) -> u64 {
    ((sample_rate * 60) as f32 / bpm as f32).round() as u64
}