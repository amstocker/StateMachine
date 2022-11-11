use std::fmt::Debug;

use dasp::{Sample, sample::{FromSample, ToSample}};
use cpal::{Device, StreamConfig, SampleFormat};
use cpal::traits::{DeviceTrait, HostTrait};

use crate::application::Float;

pub struct OutputConfig {
    pub device: Device,
    pub channels: usize,
    pub output_channels: (usize, usize),
    pub sample_rate: usize,
    pub sample_format: SampleFormat,
    pub stream_config: StreamConfig
}

impl Default for OutputConfig {
    fn default() -> Self {
        let host = cpal::default_host();
        let device = host.default_output_device().unwrap();
        let supported_config = device.default_output_config().unwrap();
        
        Self { 
            device,
            channels: supported_config.channels() as usize,
            output_channels: (0, 1),
            sample_rate: supported_config.sample_rate().0 as usize,
            sample_format: supported_config.sample_format(),
            stream_config: supported_config.into()
        }
    }
}

pub trait OutputSample:
    Sample + cpal::Sample
    + ToSample<i16> + FromSample<i16>
    + ToSample<u16> + FromSample<u16>
    + ToSample<f32> + FromSample<f32>
    + ToSample<i32> + FromSample<i32>
    + ToSample<Float> + FromSample<Float>
    + std::ops::AddAssign
    {}

impl<S> OutputSample for S where S:
    Sample + cpal::Sample
    + ToSample<i16> + FromSample<i16>
    + ToSample<u16> + FromSample<u16>
    + ToSample<f32> + FromSample<f32>
    + ToSample<i32> + FromSample<i32>
    + ToSample<Float> + FromSample<Float>
    + std::ops::AddAssign
    {}


// TODO: cpal has a FrameCount type
pub type Frames = usize;

#[derive(Debug, Clone, Copy)]
pub struct StereoFrame<S>(pub S, pub S) where S: OutputSample;

impl<S> StereoFrame<S> where S: OutputSample {
    #[inline]
    pub fn zero() -> StereoFrame<S> {
        StereoFrame(S::EQUILIBRIUM, S::EQUILIBRIUM)
    }

    pub fn get_channel(&self, channel: StereoChannel) -> S {
        match channel {
            StereoChannel::Left => self.left(),
            StereoChannel::Right => self.right()
        }
    }

    #[inline]
    pub fn left(&self) -> S {
        self.0
    }

    #[inline]
    pub fn right(&self) -> S {
        self.1
    }
}

impl<S> std::ops::AddAssign for StereoFrame<S> where S: OutputSample {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
        self.1 += rhs.1;
    }
}

pub trait StereoFrameGenerator<S> where S: OutputSample {
    fn next_frame(&mut self) -> StereoFrame<S>;
}

pub enum StereoChannel {
    Left,
    Right
}

pub struct MonoToStereoFrame<I> where I: Iterator {
    iterator: I,
}

impl<I> MonoToStereoFrame<I> where I: Iterator {
    pub fn new(iterator: I) -> Self {
        Self {
            iterator,
        }
    }
}

impl<I> Iterator for MonoToStereoFrame<I> where I: Iterator, I::Item: OutputSample {
    type Item = StereoFrame<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(sample) = self.iterator.next() {
            return Some(StereoFrame(sample, sample));
        }
        None
    }
}
