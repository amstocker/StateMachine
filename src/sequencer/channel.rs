pub const MAX_CLIPS_PER_CHANNEL: usize = 32;
pub const MAX_JUNCTIONS_PER_CHANNEL: usize = 32;

#[derive(Default)]
pub struct Channel {
    clips: [Clip; MAX_CLIPS_PER_CHANNEL],
    junctions: [Junction; MAX_JUNCTIONS_PER_CHANNEL],
    currently_playing: bool,
    playhead: u64,
}

#[derive(Default)]
pub struct Clip {
    enabled: bool,
    sound_index: usize,

    // Start and end point in terms of realtime frames
    channel_start: u64,
    channel_end: u64,

    // Start and end point in sample
    // (can be backwards if frame_end < frame_start)
    // this means dynamic interpolation will be necessary
    frame_start: u64,
    frame_end: u64,
}

pub enum JunctionType {
    Jump {
        destination_channel: usize,
        destination_frame: u64
    },
    Split {
        destination_channel: usize,
        destination_frame: u64
    },
    Reflect,
    Stop,
    Empty
}

impl Default for JunctionType {
    fn default() -> Self {
        JunctionType::Empty
    }
}

#[derive(Default)]
pub struct Junction {
    enabled: bool,
    junction_type: JunctionType,
}
