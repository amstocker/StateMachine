use crate::sequencer::NodeIndex;


#[derive(Debug, Clone, Copy)]
pub enum SequencerControlMessage {
    EnableSound(NodeIndex),
    DisableSound(NodeIndex),
    PlaySoundOnce(NodeIndex),
    IncrSoundIndex(NodeIndex),
    DecrSoundIndex(NodeIndex)
}