use std::rc::Rc;

use dasp::{Sample, sample::{FromSample, ToSample}};

use crate::sound::{Sound, SoundID};
use crate::interpolator::InterpolatorFloat;


pub struct OutputFormat {
    pub channels: u16,
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


pub type Samples = usize;

pub enum StereoChannel {
    Left,
    Right
}

pub struct MonoToStereo<I> where I: Iterator {
    iterator: I,
    buffer: Option<I::Item>,
    first: bool
}

impl<I> MonoToStereo<I> where I: Iterator {
    pub fn new(iterator: I) -> Self {
        Self {
            iterator,
            buffer: None,
            first: true
        }
    }
}

impl<I> Iterator for MonoToStereo<I> where I: Iterator, I::Item: Copy {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.first {
            self.buffer = self.iterator.next();
            self.first = false;
            self.buffer
        } else {
            self.first = true;
            self.buffer
        }
    }
}


pub struct StereoOutput<I> {
    iterator: I,
    num_channels: u16,
    output_channels: (u16, u16),
    current_channel: u16,
    next_channel: StereoChannel
}

impl<I> StereoOutput<I> where I: Iterator {
    pub fn new(iterator: I, num_channels: u16, output_channels: (u16, u16)) -> Self {
        assert!(output_channels.0 != output_channels.1);
        Self {
            iterator,
            num_channels,
            output_channels: (output_channels.0 % num_channels, output_channels.1 % num_channels),
            current_channel: 0,
            next_channel: StereoChannel::Left
        }
    }
}

impl<I> Iterator for StereoOutput<I> where I: Iterator, I::Item: Sample {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.current_channel = (self.current_channel + 1) % self.num_channels;
        if (self.current_channel == self.output_channels.0) || 
           (self.current_channel == self.output_channels.1)
        {
            self.next_channel = match self.next_channel {
                StereoChannel::Left => StereoChannel::Right,
                StereoChannel::Right => StereoChannel::Left
            };
            self.iterator.next()
        } else {
            Some(Self::Item::EQUILIBRIUM)
        }
    }
}


struct SoundNode<S> where S: OutputSample {
    index: SoundID,
    sound: Rc<Sound<S>>, // actually don't need this as long as we reference SoundBank
    samples_until_trigger: Samples,
    is_playing: bool,
    current_sample_index: Samples,
    current_sample: S
}

impl<S> SoundNode<S> where S: OutputSample {

    #[inline]
    pub fn current_sample(&self) -> Option<S> {
        self.sound.data.get(self.current_sample_index as usize).copied()
    }

    pub fn increment(&mut self) {
        self.samples_until_trigger -= 1;
        self.current_sample_index += 1;
        if self.current_sample_index >= self.sound.data.len() {
            self.is_playing = false;
        }
    }
}

struct AudioOutput<S> where S: OutputSample {
    samples_count: Samples,
    sounds: [Option<SoundNode<S>>; 16],
}

impl<S> Iterator for AudioOutput<S> where S: OutputSample {
    type Item = S;

    fn next(&mut self) -> Option<Self::Item> {
        let mut out_sample = S::EQUILIBRIUM;

        for i in 0..16 {
            if let Some(node) = &self.sounds[i] {
                if node.is_playing {
                    out_sample += node.current_sample().unwrap();
                }
            }
        }

        Some(out_sample)
    }
}

struct AudioOutputController {

}
