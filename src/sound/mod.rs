use std::{io::Read, path::Path};

use dasp::Sample;
use hound::{SampleFormat, WavReader};

mod interpolator;
mod output;
mod sound_bank;

pub use interpolator::*;
pub use output::*;
pub use sound_bank::*;

pub const MAX_SOUNDS: usize = 32;
pub type Float = f32;

#[derive(Debug)]
pub struct Sound<S: OutputSample> {
    pub metadata: SoundMetadata,
    pub data: Box<[StereoFrame<S>]>,
}

#[derive(Debug, Clone)]
pub struct SoundMetadata {
    pub name: String,
    pub length: usize,
    // TODO: downsampled waveform
}

#[derive(Debug)]
pub enum SoundLoadError {
    Wav(hound::Error),
    UnsupportedFormat(hound::WavSpec),
    InvalidSampleRate,
    NonFiniteSample,
    InvalidRatio(InvalidRatio),
}

impl std::fmt::Display for SoundLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Wav(error) => write!(f, "cannot decode WAV: {error}"),
            Self::UnsupportedFormat(spec) => write!(f,
                "unsupported WAV format: {} channels, {}-bit {:?}; expected mono 16-bit PCM or 32-bit float",
                spec.channels, spec.bits_per_sample, spec.sample_format),
            Self::InvalidSampleRate => f.write_str("sample rates must be nonzero"),
            Self::NonFiniteSample => f.write_str("WAV contains a non-finite sample"),
            Self::InvalidRatio(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for SoundLoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Wav(error) => Some(error),
            Self::InvalidRatio(error) => Some(error),
            _ => None,
        }
    }
}

impl From<hound::Error> for SoundLoadError {
    fn from(error: hound::Error) -> Self {
        Self::Wav(error)
    }
}

impl<S: OutputSample> Sound<S> {
    pub fn from_wav_file(
        path: impl AsRef<Path>,
        target_sample_rate: usize,
    ) -> Result<Self, SoundLoadError> {
        let path = path.as_ref();
        let name = path.file_stem().unwrap_or_default().to_string_lossy();
        Self::from_wav_reader(WavReader::open(path)?, &name, target_sample_rate)
    }

    /// Device-independent import entry point. Symphonia/Rubato replace this
    /// restricted mono loader and unfiltered interpolation in phase 5.
    pub fn from_wav_reader<R: Read>(
        wav: WavReader<R>,
        name: &str,
        target_sample_rate: usize,
    ) -> Result<Self, SoundLoadError> {
        let spec = wav.spec();
        if spec.sample_rate == 0 || target_sample_rate == 0 {
            return Err(SoundLoadError::InvalidSampleRate);
        }
        if spec.channels != 1
            || !matches!(
                (spec.sample_format, spec.bits_per_sample),
                (SampleFormat::Int, 16) | (SampleFormat::Float, 32)
            )
        {
            return Err(SoundLoadError::UnsupportedFormat(spec));
        }

        // Decode completely before resampling, so malformed/truncated input
        // cannot be hidden by an iterator that skips source frames.
        let samples: Vec<Float> = match spec.sample_format {
            SampleFormat::Float => wav.into_samples::<f32>().collect::<Result<_, _>>()?,
            SampleFormat::Int => wav
                .into_samples::<i16>()
                .map(|sample| sample.map(|s| s.to_sample::<Float>()))
                .collect::<Result<_, _>>()?,
        };
        if samples.iter().any(|sample| !sample.is_finite()) {
            return Err(SoundLoadError::NonFiniteSample);
        }
        let interpolated = LinearInterpolator::new(
            samples.into_iter(),
            target_sample_rate as f64 / spec.sample_rate as f64,
        )
        .map_err(SoundLoadError::InvalidRatio)?;
        let data: Box<[_]> =
            MonoToStereoFrame::new(interpolated.map(|s| s.to_sample::<S>())).collect();
        Ok(Self {
            metadata: SoundMetadata {
                name: name.to_owned(),
                length: data.len(),
            },
            data,
        })
    }
}
