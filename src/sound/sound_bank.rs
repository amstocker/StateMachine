use rtrb::{RingBuffer, Producer, Consumer};

use crate::sound::{Sound, SoundMetadata, OutputSample, StereoFrame, MAX_SOUNDS};


enum SoundBankControlMessage<S> where S: OutputSample {
    Set {
        index: usize,
        sound: Option<Sound<S>>
    }
}

pub struct SoundBankController<S> where S: OutputSample {
    pub metadata: [Option<SoundMetadata>; MAX_SOUNDS],
    producer: Producer<SoundBankControlMessage<S>>
}

impl<S> SoundBankController<S> where S: OutputSample {
    pub fn add_sound(&mut self, sound: Sound<S>) {
        for (i, slot) in &mut self.metadata.iter_mut().enumerate() {
            if slot.is_none() {
                let metadata = sound.metadata.clone();
                *slot = Some(metadata);
                self.producer.push(SoundBankControlMessage::Set {
                    index: i,
                    sound: Some(sound)
                }).unwrap();
                return;
            }
        }
    }

    pub fn get(&self, index: usize) -> Option<&SoundMetadata> {
        let slot = self.metadata.get(index)?;
        if let Some(sound_metadata) = slot {
            Some(&sound_metadata)
        } else {
            None
        }
    }
}

pub struct SoundBankIndex {
    pub source_index: usize,
    pub frame_index: usize
}

pub struct SoundBank<S> where S: OutputSample {
    sounds: [Option<Sound<S>>; MAX_SOUNDS],
    consumer: Consumer<SoundBankControlMessage<S>>
}

impl<S> SoundBank<S> where S: OutputSample {
    pub fn new(sounds: Vec<Sound<S>>) -> (SoundBankController<S>, SoundBank<S>) {
        let (producer, consumer) = RingBuffer::new(MAX_SOUNDS);
        
        let mut sound_bank_metadata = SoundBankController {
            metadata: Default::default(),
            producer
        };
        let mut sound_bank = SoundBank {
            sounds: Default::default(),
            consumer
        };
        
        for sound in sounds {
            sound_bank_metadata.add_sound(sound);
        }
        sound_bank.update();

        (sound_bank_metadata, sound_bank)
    }

    pub fn update(&mut self) {
        while let Ok(item) = self.consumer.pop() {
            use SoundBankControlMessage::*;
            match item {
                Set { index, sound } => {
                    self.sounds[index] = sound;
                }
            }
        }
    }

    pub fn get_frame(&self, index: SoundBankIndex) -> Option<StereoFrame<S>> {
        if let Some(sound) = &self.sounds[index.source_index] {
            return sound.data.get(index.frame_index).copied();
        }
        None
    }
}

