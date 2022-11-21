use crate::sequencer::{Clip, Junction, Playhead, NUM_CHANNELS};


#[derive(Debug, Clone, Copy)]
pub struct SequencerIndex {
    pub channel: usize,
    pub index: usize
}

#[derive(Debug, Clone, Copy)]
pub enum SequencerControlMessage {
    SyncClip {
        sequencer_index: SequencerIndex,
        clip: Clip
    },
    SyncJunction {
        sequencer_index: SequencerIndex,
        junction: Junction
    },
    SyncPlayhead {
        channel_index: usize,
        playhead: Playhead
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SequencerState {
    pub playheads: [Playhead; NUM_CHANNELS],
    pub total_frames_processed: u64
}

#[derive(Debug, Clone, Copy)]
pub enum SequencerEvent {
    Tick(SequencerState)
}