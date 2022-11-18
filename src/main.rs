use bevy::prelude::*;

mod application;
mod fonts;
mod utils;
mod sound;
mod interpolator;
mod sequencer;
mod output;
mod config;
mod grid;

use crate::application::*;
use crate::sound::Sound;
use crate::config::Config;

fn main() {
    let config = Config::default();

    let files = [
        "assets/samples/kick.wav",
        "assets/samples/snare.wav",
        "assets/samples/hihat.wav"
    ];
    // for path in files {
    //     config.sounds.push(Sound::from_wav_file(path, &config.output));
    // }

    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(config)
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .add_startup_system(start_sequencer)
        .add_system(do_something_with_sequencer)
        .run();
}