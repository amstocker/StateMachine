mod application;
mod fonts;
mod utils;
mod sound;
mod interpolator;
mod sequencer;
mod output;
mod config;

use iced::{Application, Settings};
use sound::Sound;

use crate::application::Instrument;
use crate::config::Config;


fn main() -> iced::Result {
    let mut config = Config::default();

    let files = [
        "assets/samples/kick.wav",
        "assets/samples/snare.wav",
        "assets/samples/hihat.wav"
    ];
    for path in files {
        config.sounds.push(Sound::from_wav_file(path, &config.output));
    }

    Instrument::run(Settings {
        flags: config,
        ..Settings::default()
    })
}