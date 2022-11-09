#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
use std::thread;
use std::time::Duration;
use std::sync::atomic::Ordering::SeqCst;

use assert_no_alloc::*;
use dasp::Sample;
use hound;
use cpal::{SampleRate, StreamConfig};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use output::OutputFormat;
use ringbuf::LocalRb;
use sequencer::Sequencer;

mod utils;
mod sound;
mod interpolator;
mod sequencer;
mod output;
use crate::sound::*;
use crate::interpolator::*;
use crate::output::{MonoToStereoFrame, stereo_to_output_frame};


// assert_no_alloc
#[cfg(debug_assertions)]
#[global_allocator]
static A: AllocDisabler = AllocDisabler;


fn main() {
    let host = cpal::default_host();
    let device = host.default_output_device().unwrap();

    let supported_config = device.default_output_config().unwrap();
    let sample_format = supported_config.sample_format();
    let config: StreamConfig = supported_config.into();

    let num_channels = config.channels as usize;
    let SampleRate(sample_rate) = config.sample_rate;
    let format = OutputFormat {
        channels: num_channels,
        sample_rate: sample_rate as f32,
        sample_format
    };

    let sound: Sound<f32> = Sound::from_wav_file("samples/hihat.wav", &format);
    let mut sound_bank = SoundBank::new();
    sound_bank.add_sound_at_index(0, sound);

    let (controller, mut sequencer) = Sequencer::new_with_sound_bank(sound_bank);

    let stream = device.build_output_stream(
        &config,
        move |data: &mut [f32], _| {
            assert_no_alloc(|| {
                for out_frame in data.chunks_mut(format.channels) {
                    stereo_to_output_frame(out_frame, sequencer.next_frame(), num_channels, (0, 1));
                }
            });
        },
        move |err| {
            println!{"{}", err};
        },
    ).unwrap();
    stream.play().unwrap();

    if let Some(node) = controller.nodes.get(0) {
        node.enabled.store(true, SeqCst);
        node.is_playing.store(true, SeqCst);
    }

    thread::sleep(Duration::from_millis(1000));
}


