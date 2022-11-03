#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
mod app;
mod sound;
mod playback;
mod ui;
mod interpolator;


use std::thread;
use std::time::Duration;

use assert_no_alloc::*;
use crossbeam_channel::Iter;
use hound::{self, WavIntoSamples, WavReader};
use ringbuf::SharedRb;
use cpal::{Device, Data, SampleFormat, StreamConfig};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use dasp::Sample;
use dasp_signal::{Signal, from_interleaved_samples_iter, from_iter};
use dasp_interpolate::sinc::Sinc;
use dasp_ring_buffer::Fixed;

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

fn print_supported_configs(device: &Device) {
    println!("Supported Configs:");
    let configs = device.supported_output_configs().unwrap();
    for config in configs {
        println!("\t{:?}", config);
    }
}

fn main() {
    let v: Vec<i16> = vec![0, 150, 0, -150, 100];
    println!("original: {:?}", v);
    let interpolated: Vec<i16> = LinearInterpolator::new(v.into_iter(), 1.5).collect();
    println!("interpolated: {:?}", interpolated);

    let v: Vec<i16> = vec![0];
    println!("original: {:?}", v);
    let interpolated: Vec<i16> = LinearInterpolator::new(v.into_iter(), 1.5).collect();
    println!("interpolated: {:?}", interpolated);

    let v: Vec<i16> = vec![0, 100, 200, 0];
    println!("original: {:?}", v);
    let interpolated: Vec<i16> = LinearInterpolator::new(v.into_iter(), 1.5).collect();
    println!("interpolated: {:?}", interpolated);
    

    let host = cpal::default_host();
    let device = host.default_output_device().unwrap();
    let config: StreamConfig = device.default_output_config().unwrap().into();
    let cpal::SampleRate(sample_rate) = config.sample_rate;
    println!("target sample rate: {}", sample_rate as f64);

    let wav = hound::WavReader::open("samples/snare.wav").unwrap();
    let spec = wav.spec();
    println!("source sample rate: {}", spec.sample_rate as f64);

    let mut samples = MonoToStereo::new(
        wav.into_samples::<i16>().map(|r| r.unwrap())
    );

    let stream = device.build_output_stream(
        &config,
        move |data: &mut [f32], _| {
            assert_no_alloc(|| {
                for out_sample in data.iter_mut() {
                    if let Some(sample) = samples.next() {
                        *out_sample = cpal::Sample::from(&sample);
                    }
                }
            });
        },
        move |err| {
            println!{"{}", err};
        },
    ).unwrap();
    stream.play().unwrap();

    thread::sleep(Duration::from_millis(1000));
}


