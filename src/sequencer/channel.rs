pub const MAX_CLIPS_PER_CHANNEL: usize = 32;
pub const MAX_JUNCTIONS_PER_CHANNEL: usize = 32;

#[derive(Default)]
pub struct Channel {
    clips: [Clip; MAX_CLIPS_PER_CHANNEL],
    junctions: [Junction; MAX_JUNCTIONS_PER_CHANNEL],
    currently_playing: bool,
    playhead_location: u64,
    length: u64
}

#[derive(Default)]
pub enum Direction {
    #[default] Right,
    Left
}

#[derive(Default)]
pub struct Clip {
    enabled: bool,
    source_index: usize,

    // Start and end point in terms of realtime frames
    channel_frame_start: u64,
    channel_frame_end: u64,

    // Start and end point in sound source
    // (can be backwards if frame_end < frame_start)
    // this means dynamic interpolation will be necessary
    source_frame_start: u64,
    source_frame_end: u64,
}

#[derive(Default)]
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
    #[default] Empty
}

#[derive(Default)]
pub struct Junction {
    enabled: bool,
    junction_type: JunctionType,
}
