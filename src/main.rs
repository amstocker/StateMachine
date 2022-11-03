#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
mod app;
mod sound;
mod playback;
mod ui;
mod interpolator;
mod utils;


use assert_no_alloc::*;
use crossbeam_channel::unbounded;
use hound;
use cpal::{SampleRate, StreamConfig};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use interpolator::LinearInterpolator;


// assert_no_alloc
#[cfg(debug_assertions)]
#[global_allocator]
static A: AllocDisabler = AllocDisabler;


struct MonoToStereo<I> where I: Iterator {
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


fn main() {
    let host = cpal::default_host();
    let device = host.default_output_device().unwrap();
    let config: StreamConfig = device.default_output_config().unwrap().into();
    let SampleRate(sample_rate) = config.sample_rate;

    let wav = hound::WavReader::open("samples/kick.wav").unwrap();
    let spec = wav.spec();

    let mut samples = MonoToStereo::new(
        LinearInterpolator::new(
            wav.into_samples::<i16>().map(|r| r.unwrap()),
            (sample_rate as f64) / (spec.sample_rate as f64)
        )
    );

    let (sender, receiver) = unbounded();

    let stream = device.build_output_stream(
        &config,
        move |data: &mut [f32], _| {
            // assert_no_alloc(|| {
                for out_sample in data.iter_mut() {
                    if let Some(sample) = samples.next() {
                        *out_sample = cpal::Sample::from(&sample);
                    } else {
                        sender.send(()).unwrap();
                    }
                }
            // });
        },
        move |err| {
            println!{"{}", err};
        },
    ).unwrap();
    stream.play().unwrap();

    receiver.recv().unwrap();
}


