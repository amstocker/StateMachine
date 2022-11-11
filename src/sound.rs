use std::path::PathBuf;

use dasp::Sample;
use hound::SampleFormat;
use hound::WavReader;
use rtrb::{RingBuffer, Consumer, Producer};

use crate::application::Float;
use crate::interpolator::LinearInterpolator;
use crate::output::MonoToStereoFrame;
use crate::output::OutputConfig;
use crate::output::OutputSample;
use crate::output::Frames;
use crate::output::StereoFrame;


pub const MAX_SOUNDS: usize = 32;

enum SoundBankControl<S> where S: OutputSample {
    Set {
        index: usize,
        sound: Option<Sound<S>>
    }
}

pub struct SoundBankMetadata<S> where S: OutputSample {
    pub metadata: [Option<SoundMetadata>; MAX_SOUNDS],
    producer: Producer<SoundBankControl<S>>
}

impl<S> SoundBankMetadata<S> where S: OutputSample {
    pub fn add_sound(&mut self, sound: Sound<S>) -> Option<SoundMetadata> {
        for (i, slot) in &mut self.metadata.iter_mut().enumerate() {
            if slot.is_none() {
                let metadata = SoundMetadata {
                    name: sound.name.clone(),
                    length: sound.data.len(),
                    index: i
                };
                *slot = Some(metadata.clone());
                self.producer.push(SoundBankControl::Set {
                    index: i,
                    sound: Some(sound)
                }).unwrap();
                return Some(metadata);
            }
        }
        None
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

pub struct SoundBank<S> where S: OutputSample {
    sounds: [Option<Sound<S>>; MAX_SOUNDS],
    consumer: Consumer<SoundBankControl<S>>
}

impl<S> SoundBank<S> where S: OutputSample {
    pub fn new(sounds: Vec<Sound<S>>) -> (SoundBankMetadata<S>, SoundBank<S>) {
        let (producer, consumer) = RingBuffer::new(MAX_SOUNDS);

        let mut sound_bank_metadata = SoundBankMetadata {
            metadata: Default::default(),
            producer
        };
        let mut sound_bank = SoundBank {
            sounds: Default::default(),
            consumer
        };

        for sound in sounds {
            sound_bank_metadata.add_sound(sound).unwrap();
        }
        sound_bank.update();

        (sound_bank_metadata, sound_bank)
    }

    pub fn update(&mut self) {
        while let Ok(item) = self.consumer.pop() {
            use SoundBankControl::*;
            match item {
                Set { index, sound } => {
                    self.sounds[index] = sound;
                }
            }
        }
    }

    pub fn get_frame(&self, index: usize, frame_index: Frames) -> Option<StereoFrame<S>> {
        if let Some(sound) = &self.sounds[index] {
            return sound.data.get(frame_index).copied();
        }
        None
    }
}


pub struct Sound<S> where S: OutputSample {
    pub name: String,
    pub data: Vec<StereoFrame<S>>
}

#[derive(Debug, Clone)]
pub struct SoundMetadata {
    pub name: String,
    pub length: usize,
    pub index: usize
    // TODO: downsampled waveform
}

impl<S> Sound<S> where S: OutputSample {
    pub fn new(name: String, data: Vec<StereoFrame<S>>) -> Self {
        Sound {
            name,
            data
        }
    }

    pub fn from_wav_file(path: &str, format: &OutputConfig) -> Sound<S> {
        let path = PathBuf::from(path);
        let name = path.file_stem().unwrap()
                               .to_str().unwrap()
                               .to_owned();
        let wav = WavReader::open(path).unwrap();
        let sample_rate = wav.spec().sample_rate;

        // TODO: handle stereo wav files
        let data: Vec<StereoFrame<S>> = match wav.spec().sample_format {
            SampleFormat::Float => {
                let data = wav.into_samples::<f32>()
                   .map(|r| r.unwrap())
                   .map(|s| s.to_sample::<S>());
                MonoToStereoFrame::new(LinearInterpolator::new(
                    data,
                    (format.sample_rate as Float) / (sample_rate as Float)
                )).collect()
            },
            SampleFormat::Int => {
                let data = wav.into_samples::<i16>()
                   .map(|r| r.unwrap())
                   .map(|s| s.to_sample::<S>());
                MonoToStereoFrame::new(LinearInterpolator::new(
                    data,
                    (format.sample_rate as Float) / (sample_rate as Float)
                )).collect()
            }
        };
        
        Sound::new(name, data)
    }

}

