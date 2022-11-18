mod application;
mod sound;
mod sequencer;
mod ui;
mod config;

use crate::application::run;
use crate::config::Config;
use crate::sound::Sound;


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

    run(config);
}