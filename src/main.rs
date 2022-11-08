#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
use std::thread;
use std::time::Duration;

use assert_no_alloc::*;
use dasp::Sample;
use hound;
use cpal::{SampleRate, StreamConfig};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

mod utils;
mod sound;
mod interpolator;
mod output;
use crate::sound::*;
use crate::interpolator::*;
use crate::output::{MonoToStereo, StereoOutput};


// assert_no_alloc
#[cfg(debug_assertions)]
#[global_allocator]
static A: AllocDisabler = AllocDisabler;


fn main() {
    let host = cpal::default_host();
    let device = host.default_output_device().unwrap();

    let supported_config = device.default_output_config().unwrap();
    let config: StreamConfig = supported_config.into();

    let num_channels = config.channels;
    let SampleRate(sample_rate) = config.sample_rate;

    let wav = hound::WavReader::open("samples/hihat.wav").unwrap();
    let spec = wav.spec();

    let samples = LinearInterpolator::new(
        wav.into_samples::<i16>().map(|r| r.unwrap()),
        (sample_rate as InterpolatorFloat) / (spec.sample_rate as InterpolatorFloat)
    );

    let mut output = StereoOutput::new(
        MonoToStereo::new(samples),
        num_channels,
        (1, 2) // Weirdly needs to be (1, 2) on MOTU interface?
    );

    let stream = device.build_output_stream(
        &config,
        move |data: &mut [f32], _| {
            assert_no_alloc(|| {
                for out_sample in data.iter_mut() {
                    if let Some(sample) = output.next() {
                        *out_sample = sample.to_sample::<f32>();
                    }
                }
            });
        },
        move |err| {
            println!{"{}", err};
        },
    ).unwrap();
    stream.play().unwrap();

    thread::sleep(Duration::from_millis(500));
}


