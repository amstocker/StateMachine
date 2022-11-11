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

pub struct SoundBankMeta<S> where S: OutputSample {
    pub meta_data: [Option<SoundMeta>; MAX_SOUNDS],
    producer: Producer<SoundBankControl<S>>
}

impl<S> SoundBankMeta<S> where S: OutputSample {
    pub fn add_sound(&mut self, sound: Sound<S>) -> Option<SoundMeta> {
        for (i, slot) in &mut self.meta_data.iter_mut().enumerate() {
            if slot.is_none() {
                let meta = SoundMeta {
                    name: sound.name.clone(),
                    length: sound.data.len(),
                    index: i
                };
                *slot = Some(meta.clone());
                self.producer.push(SoundBankControl::Set {
                    index: i,
                    sound: Some(sound)
                }).unwrap();
                return Some(meta);
            }
        }
        None
    }

    pub fn get_sound_meta(&self, index: usize) -> Option<&SoundMeta> {
        let slot = self.meta_data.get(index)?;
        if let Some(sound_meta) = slot {
            Some(&sound_meta)
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
    pub fn new() -> (SoundBankMeta<S>, SoundBank<S>) {
        let (producer, consumer) = RingBuffer::new(MAX_SOUNDS);

        let sound_bank_meta = SoundBankMeta {
            meta_data: Default::default(),
            producer
        };
        let sound_bank = SoundBank {
            sounds: Default::default(),
            consumer
        };
        (sound_bank_meta, sound_bank)
    }

    pub fn update(&mut self) {
        if let Ok(item) = self.consumer.pop() {
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
pub struct SoundMeta {
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

