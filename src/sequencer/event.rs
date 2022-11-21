use crate::sequencer::NUM_CHANNELS;


#[derive(Debug, Clone, Copy)]
pub enum SequencerControlMessage {

}

#[derive(Debug, Default, Clone, Copy)]
pub struct PlayheadState {
    enabled: bool,
    location: u64
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SequencerState {
    pub playheads: [PlayheadState; NUM_CHANNELS],
    pub total_frames_processed: u64
}

#[derive(Debug, Clone, Copy)]
pub enum SequencerEvent {
    Tick(SequencerState)
}