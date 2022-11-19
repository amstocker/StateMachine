mod sound;
mod sequencer;
mod ui;
mod config;
mod instrument;

use crate::config::InstrumentConfig;
use crate::instrument::Instrument;
use crate::ui::Application;
use crate::sound::Sound;


fn main() {
    let mut config = InstrumentConfig::default();

    let files = [
        "assets/samples/kick.wav",
        "assets/samples/snare.wav",
        "assets/samples/hihat.wav"
    ];
    config.sounds = files.map(|path| Sound::from_wav_file(path, &config.output)).into();

    Instrument::run(config);
}