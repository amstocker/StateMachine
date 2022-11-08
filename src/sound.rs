use std::path::PathBuf;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use dasp::Sample;
use hound::SampleFormat;
use hound::WavReader;

use crate::interpolator::InterpolatorFloat;
use crate::interpolator::LinearInterpolator;
use crate::output::OutputFormat;
use crate::output::OutputSample;
use crate::output::Samples;


const MAX_SOUNDS: usize = 256;
pub type SoundID = usize;

pub struct SoundBank<S>([Option<Sound<S>>; MAX_SOUNDS]) where S: OutputSample;

impl<S> SoundBank<S> where S: OutputSample {
    pub fn get_sample(&self, id: SoundID, index: Samples) -> Option<S> {
        if let Some(sound) = &self.0[id] {
            sound.data.get(index).cloned()
        } else {
            None
        }
    }
}


fn generate_id() -> SoundID {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    COUNTER.fetch_add(1, Ordering::SeqCst)
}


pub struct Sound<S> where S: OutputSample {
    pub id: SoundID,
    pub name: String,
    pub data: Vec<S>
}

impl<S> Sound<S> where S: OutputSample {
    pub fn new(name: String, data: Vec<S>) -> Self {
        Sound {
            id: generate_id(),
            name,
            data
        }
    }

    pub fn from_wav_file(path: &str, format: OutputFormat) -> Sound<S> {
        let path = PathBuf::from(path);
        let name = path.file_stem().unwrap()
                               .to_str().unwrap()
                               .to_owned();
        let wav = WavReader::open(path).unwrap();
        let sample_rate = wav.spec().sample_rate;
        let data = match wav.spec().sample_format {
            SampleFormat::Float => {
                let data = wav.into_samples::<f32>()
                   .map(|r| r.unwrap())
                   .map(|s| s.to_sample::<S>());
                LinearInterpolator::new(
                    data,
                    (format.sample_rate as InterpolatorFloat) / (sample_rate as InterpolatorFloat)
                ).collect()
            },
            SampleFormat::Int => {
                let data = wav.into_samples::<i16>()
                   .map(|r| r.unwrap())
                   .map(|s| s.to_sample::<S>());
                LinearInterpolator::new(
                    data,
                    (format.sample_rate as InterpolatorFloat) / (sample_rate as InterpolatorFloat)
                ).collect()
            }
        };
        Sound::new(name, data)
    }

}

