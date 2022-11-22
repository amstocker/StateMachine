use std::path::PathBuf;

use dasp::Sample;
use hound::SampleFormat;
use hound::WavReader;

mod sound_bank;
mod output;
mod interpolator;

pub use sound_bank::*;
pub use output::*;
pub use interpolator::*;


pub const MAX_SOUNDS: usize = 32;

pub type Float = f32;


pub struct Sound<S> where S: OutputSample {
    pub metadata: SoundMetadata,
    pub data: Box<[StereoFrame<S>]>
}

#[derive(Debug, Clone)]
pub struct SoundMetadata {
    pub name: String,
    pub length: usize,
    // TODO: downsampled waveform
}

impl<S> Sound<S> where S: OutputSample {
    pub fn from_wav_file(path: &str, format: &OutputConfig) -> Sound<S> {
        let path = PathBuf::from(path);
        let name = path.file_stem().unwrap()
                               .to_str().unwrap()
                               .to_owned();
        let wav = WavReader::open(path).unwrap();
        let sample_rate = wav.spec().sample_rate;

        // TODO: handle stereo wav files
        // TODO: better interpolation
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
        
        let metadata = SoundMetadata {
            name,
            length: data.len(),
        };
        Sound {
            metadata,
            data: data.into_boxed_slice()
        }
    }
}

