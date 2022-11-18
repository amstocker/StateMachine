use std::fmt::Debug;

use dasp::{Sample, sample::{FromSample, ToSample}};
use cpal::{Stream, StreamConfig, SampleFormat};
use cpal::traits::{HostTrait, DeviceTrait, StreamTrait};
use assert_no_alloc::*;

use crate::sound::Float;

#[cfg(debug_assertions)]
#[global_allocator]
static A: AllocDisabler = AllocDisabler;


pub struct OutputConfig {
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
            channels: supported_config.channels() as usize,
            output_channels: (0, 1),
            sample_rate: supported_config.sample_rate().0 as usize,
            sample_format: supported_config.sample_format(),
            stream_config: supported_config.config()
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
{
    // Trait alias
}

impl<S> OutputSample for S where S:
    Sample + cpal::Sample
    + ToSample<i16> + FromSample<i16>
    + ToSample<u16> + FromSample<u16>
    + ToSample<f32> + FromSample<f32>
    + ToSample<i32> + FromSample<i32>
    + ToSample<Float> + FromSample<Float>
    + std::ops::AddAssign
{
    // Trait alias
}

pub enum StereoChannel {
    Left,
    Right
}

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


pub struct Output {
    output_config: OutputConfig,
    output_stream: Option<Stream>
}

impl Output {
    pub fn new(output_config: OutputConfig) -> Self {
        Self {
            output_config,
            output_stream: None
        }
    }

    pub fn start<T: 'static + StereoFrameGenerator<Float> + Send>(&mut self, frame_generator: T) {
        use SampleFormat::*;
        let stream = match self.output_config.sample_format {
            I16 => self.build_stream::<i16, _>(frame_generator),
            U16 => self.build_stream::<u16, _>(frame_generator),
            F32 => self.build_stream::<f32, _>(frame_generator)
        };
        stream.play().unwrap();

        self.output_stream = Some(stream);
    }

    pub fn build_stream<S: OutputSample, T: 'static + StereoFrameGenerator<Float> + Send>(
        &self,
        mut frame_generator: T
    ) -> Stream {
        let host = cpal::default_host();
        let device = host.default_output_device().unwrap();
        
        let channels = self.output_config.channels;
        let output_channels = self.output_config.output_channels;

        device.build_output_stream(
            &self.output_config.stream_config,
            move |data: &mut [S], _| {
                assert_no_alloc(|| {
                    for out_frame in data.chunks_mut(channels) {
                        let frame = frame_generator.next_frame();
                        for (i, out_sample) in out_frame.iter_mut().enumerate() {
                            if i == output_channels.0 % channels {
                                *out_sample = frame.left().to_sample::<S>();
                            } else if i == output_channels.1 % channels {
                                *out_sample = frame.right().to_sample::<S>();
                            } else {
                                *out_sample = S::EQUILIBRIUM;
                            }
                        } 
                    }
                });
            },
            move |err| {
                println!{"{}", err};
            },
        ).unwrap()
    }
}
