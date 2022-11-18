mod application;
mod fonts;
mod sound;
mod sequencer;
mod config;
mod grid;

use crate::sound::Sound;
use crate::application::run;
use crate::config::Config;

fn main() {
    let mut config = Config::default();

    let files = [
        "assets/samples/kick.wav",
        "assets/samples/snare.wav",
        "assets/samples/hihat.wav"
    ];
    for path in files {
        config.sounds.push(Sound::from_wav_file(path, &config.output));
    }

    run();
}