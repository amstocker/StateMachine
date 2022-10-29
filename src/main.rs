#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
mod app;
mod playback;
mod sound;
mod ui;

use app::App;
use sound::{TriggerInfo, Delay};


fn main() {
    let mut app = App::new();

    let sound_filenames = vec![
        "samples/kick.wav",
        "samples/snare.wav",
        "samples/hihat.wav"
    ];
    for filename in sound_filenames {
        app.add_sound(filename.to_string());
    }

    app.add_trigger(0, TriggerInfo {
        target: 0,
        delay: Delay::Milliseconds(500)
    });

    app.run();
}


