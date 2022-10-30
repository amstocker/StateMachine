#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
mod app;
mod sound;
mod playback;
mod ui;

use app::App;

fn main() {
    let mut app = App::new();

    let sound_filenames = vec![
        "samples/kick.wav",
        "samples/snare.wav",
        "samples/hihat.wav"
    ];

    app.run();
}


