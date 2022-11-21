use crate::sequencer::{Clip, Junction, Playhead, NUM_CHANNELS};


#[derive(Debug, Clone, Copy)]
pub struct ChannelItemIndex {
    pub channel_index: usize,
    pub item_index: usize
}

#[derive(Debug, Clone, Copy)]
pub enum SequencerControlMessage {
    SyncClip {
        index: ChannelItemIndex,
        clip: Clip
    },
    SyncJunction {
        index: ChannelItemIndex,
        junction: Junction
    },
    SyncPlayhead {
        channel_index: usize,
        playhead: Playhead
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SequencerSummary {
    pub playheads: [Playhead; NUM_CHANNELS],
    pub total_frames_processed: u64
}

#[derive(Debug, Clone, Copy)]
pub enum SequencerEvent {
    Tick(SequencerSummary)
}