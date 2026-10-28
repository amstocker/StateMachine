use rtrb::{Consumer, Producer, PushError, RingBuffer};

use crate::sound::{OutputSample, Sound, SoundMetadata, StereoFrame, MAX_SOUNDS};

// Slots are append-only until phase 4 supplies resource retirement. There is no
// replacement path that could drop an existing sample on the audio callback.
struct AddSound<S: OutputSample> {
    index: usize,
    sound: Sound<S>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundBankError {
    Full,
    QueueFull,
    EngineDisconnected,
}

impl std::fmt::Display for SoundBankError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Full => "sound bank is full",
            Self::QueueFull => "sound upload queue is full; try again",
            Self::EngineDisconnected => "audio engine is disconnected",
        })
    }
}
impl std::error::Error for SoundBankError {}

pub struct SoundBankController<S: OutputSample> {
    metadata: [Option<SoundMetadata>; MAX_SOUNDS],
    producer: Producer<AddSound<S>>,
}

impl<S: OutputSample> SoundBankController<S> {
    pub fn add_sound(&mut self, sound: Sound<S>) -> Result<usize, SoundBankError> {
        if self.producer.is_abandoned() {
            return Err(SoundBankError::EngineDisconnected);
        }
        let index = self
            .metadata
            .iter()
            .position(Option::is_none)
            .ok_or(SoundBankError::Full)?;
        let metadata = sound.metadata.clone();
        self.producer
            .push(AddSound { index, sound })
            .map_err(|PushError::Full(_)| SoundBankError::QueueFull)?;
        // Publish metadata only after the queue has accepted the asset.
        self.metadata[index] = Some(metadata);
        Ok(index)
    }

    pub fn get(&self, index: usize) -> Option<&SoundMetadata> {
        self.metadata.get(index)?.as_ref()
    }
}

pub struct SoundBankIndex {
    pub source_index: usize,
    pub frame_index: usize,
}

pub struct SoundBank<S: OutputSample> {
    sounds: [Option<Sound<S>>; MAX_SOUNDS],
    consumer: Consumer<AddSound<S>>,
}

impl<S: OutputSample> SoundBank<S> {
    pub fn new(sounds: Vec<Sound<S>>) -> Result<(SoundBankController<S>, Self), SoundBankError> {
        if sounds.len() > MAX_SOUNDS {
            return Err(SoundBankError::Full);
        }
        let (producer, consumer) = RingBuffer::new(MAX_SOUNDS);
        let mut controller = SoundBankController {
            metadata: Default::default(),
            producer,
        };
        let mut bank = Self {
            sounds: Default::default(),
            consumer,
        };
        for sound in sounds {
            controller.add_sound(sound)?;
        }
        bank.update();
        Ok((controller, bank))
    }

    pub fn update(&mut self) {
        for _ in 0..MAX_SOUNDS {
            let Ok(AddSound { index, sound }) = self.consumer.pop() else {
                break;
            };
            // The private producer only emits valid, previously unused slots.
            self.sounds[index] = Some(sound);
        }
    }

    pub fn get_frame(&self, index: SoundBankIndex) -> Option<StereoFrame<S>> {
        self.sounds
            .get(index.source_index)?
            .as_ref()?
            .data
            .get(index.frame_index)
            .copied()
    }
}
