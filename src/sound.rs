use std::path::PathBuf;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use dasp::Sample;
use hound::SampleFormat;
use hound::WavReader;

use crate::interpolator::InterpolatorFloat;
use crate::interpolator::LinearInterpolator;
use crate::output::MonoToStereoFrame;
use crate::output::OutputFormat;
use crate::output::OutputSample;
use crate::output::Frames;
use crate::output::StereoFrame;


const MAX_SOUNDS: usize = 32;

pub type SoundID = usize;

pub struct SoundBank<S>([Option<Sound<S>>; MAX_SOUNDS]) where S: OutputSample;

impl<S> SoundBank<S> where S: OutputSample {
    pub fn new() -> SoundBank<S> {
        SoundBank(Default::default())
    }

    pub fn add_sound_at_index(&mut self, index: SoundID, sound: Sound<S>) {
        self.0[index] = Some(sound);
    }

    pub fn get_frame(&self, id: SoundID, index: Frames) -> Option<StereoFrame<S>> {
        if let Some(sound) = &self.0[id] {
            return sound.data.get(index).copied();
        }
        None
    }
}


fn generate_id() -> SoundID {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    COUNTER.fetch_add(1, Ordering::SeqCst)
}


pub struct Sound<S> where S: OutputSample {
    pub id: SoundID,
    pub name: String,
    pub data: Vec<StereoFrame<S>>
}

impl<S> Sound<S> where S: OutputSample {
    pub fn new(name: String, data: Vec<StereoFrame<S>>) -> Self {
        Sound {
            id: generate_id(),
            name,
            data
        }
    }

    pub fn from_wav_file(path: &str, format: &OutputFormat) -> Sound<S> {
        let path = PathBuf::from(path);
        let name = path.file_stem().unwrap()
                               .to_str().unwrap()
                               .to_owned();
        let wav = WavReader::open(path).unwrap();
        let sample_rate = wav.spec().sample_rate;

        // need to handle stereo wav files
        let data = match wav.spec().sample_format {
            SampleFormat::Float => {
                let data = wav.into_samples::<f32>()
                   .map(|r| r.unwrap())
                   .map(|s| s.to_sample::<S>());
                MonoToStereoFrame::new(LinearInterpolator::new(
                    data,
                    (format.sample_rate as InterpolatorFloat) / (sample_rate as InterpolatorFloat)
                )).collect()
            },
            SampleFormat::Int => {
                let data = wav.into_samples::<i16>()
                   .map(|r| r.unwrap())
                   .map(|s| s.to_sample::<S>());
                MonoToStereoFrame::new(LinearInterpolator::new(
                    data,
                    (format.sample_rate as InterpolatorFloat) / (sample_rate as InterpolatorFloat)
                )).collect()
            }
        };
        Sound::new(name, data)
    }

}

