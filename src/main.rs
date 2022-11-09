#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
use assert_no_alloc::*;
use cpal::{SampleRate, StreamConfig};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use output::OutputFormat;
use sequencer::Sequencer;
use ui::UI;

mod ui;
mod utils;
mod sound;
mod interpolator;
mod sequencer;
mod output;
use crate::sound::*;
use crate::output::stereo_to_output_frame;


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

    let (mut sound_bank_meta, sound_bank) = SoundBank::new();
    
    // Pre-load with some drum sounds...
    sound_bank_meta.add_sound(Sound::from_wav_file("samples/kick.wav", &format)).unwrap();
    sound_bank_meta.add_sound(Sound::from_wav_file("samples/snare.wav", &format)).unwrap();
    sound_bank_meta.add_sound(Sound::from_wav_file("samples/hihat.wav", &format)).unwrap();

    let (controller, mut sequencer) = Sequencer::new(sound_bank);

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

    let ui = UI::new(sound_bank_meta, controller);
    ui.run();
}


