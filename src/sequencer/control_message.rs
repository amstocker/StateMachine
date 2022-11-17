

#[derive(Debug, Clone, Copy)]
pub enum SequencerControlMessage {
    EnableSound(usize),
    DisableSound(usize),
    PlaySoundOnce(usize),
    IncrSoundIndex(usize),
    DecrSoundIndex(usize)
}