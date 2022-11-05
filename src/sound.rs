use std::path::PathBuf;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use dasp::Sample;
use hound::SampleFormat;
use hound::WavReader;


pub type Float = f32;

pub type SoundID = usize;

fn generate_id() -> SoundID {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

pub struct Sound {
    id: SoundID,
    name: String,
    data: Vec<Float>,
    sample_rate: Float
}

impl Sound {
    pub fn new(name: String, data: Vec<Float>, sample_rate: Float) -> Self {
        Sound {
            id: generate_id(),
            name,
            data,
            sample_rate
        }
    }

    pub fn from_wav_file(path: &str) -> Sound {
        let path = PathBuf::from(path);
        let name = path.file_stem().unwrap()
                               .to_str().unwrap()
                               .to_owned();
        let wav = WavReader::open(path).unwrap();
        let sample_rate = wav.spec().sample_rate as Float;
        let data = match wav.spec().sample_format {
            SampleFormat::Float => {
                wav.into_samples::<f32>()
                   .map(|r| r.unwrap())
                   .map(|s| s.to_sample::<Float>())
                   .collect()
            },
            SampleFormat::Int => {
                wav.into_samples::<i32>()
                   .map(|r| r.unwrap())
                   .map(|s| s.to_sample::<Float>())
                   .collect()
            }
        };
        Sound::new(name, data, sample_rate)
    }

}

