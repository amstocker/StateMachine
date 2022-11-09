use core::num;
use std::rc::Rc;

use dasp::{Sample, sample::{FromSample, ToSample}};

use crate::sound::{Sound, SoundID};
use crate::interpolator::InterpolatorFloat;


pub struct OutputFormat {
    pub channels: usize,
    pub sample_rate: f32,
    pub sample_format: cpal::SampleFormat
}

pub trait OutputSample:
    Sample
    + ToSample<i16> + FromSample<i16>
    + ToSample<u16> + FromSample<u16>
    + ToSample<f32> + FromSample<f32>
    + ToSample<InterpolatorFloat> + FromSample<InterpolatorFloat>
    + std::ops::AddAssign
    {}

impl<S> OutputSample for S where S:
    Sample
    + ToSample<i16> + FromSample<i16>
    + ToSample<u16> + FromSample<u16>
    + ToSample<f32> + FromSample<f32>
    + ToSample<InterpolatorFloat> + FromSample<InterpolatorFloat>
    + std::ops::AddAssign
    {}

pub type Frames = usize;

#[derive(Clone, Copy)]
pub struct StereoFrame<S>(S, S) where S: OutputSample;

impl<S> StereoFrame<S> where S: OutputSample {
    #[inline]
    pub fn zero() -> StereoFrame<S> {
        StereoFrame(S::EQUILIBRIUM, S::EQUILIBRIUM)
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

pub fn stereo_to_output_frame<S: OutputSample>(
    output_frame: &mut [S],
    input_frame: StereoFrame<S>,
    num_channels: usize,
    output_channels: (usize, usize)
) {
    for (i, out_sample) in output_frame.iter_mut().enumerate() {
        if i == output_channels.0 % num_channels {
            *out_sample = input_frame.left();
        } else if i == output_channels.1 % num_channels {
            *out_sample = input_frame.right();
        } else {
            *out_sample = S::EQUILIBRIUM;
        }
    } 
}