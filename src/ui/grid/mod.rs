mod background;

pub use background::GridBackground;

use crate::sequencer::{
    Clip, MAX_CLIPS_PER_CHANNEL,
    Junction, MAX_JUNCTIONS_PER_CHANNEL
};


pub struct ClipInterface {
    model: Clip
}

pub struct JunctionInterface {
    model: Junction
}

pub struct ChannelInterface {
    clips: [ClipInterface; MAX_CLIPS_PER_CHANNEL],
    junctions: [JunctionInterface; MAX_JUNCTIONS_PER_CHANNEL]
}