#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
mod app;
mod sound;
mod playback;
mod ui;

use app::App;
use playback::{Link, Delay};

fn main() {
    let mut app = App::new();

    let sounds = vec![
        "samples/kick.wav",
        "samples/snare.wav",
        "samples/hihat.wav"
    ];

    for path in sounds {
        app.add_sound(path.to_string());
    }

    app.add_link(Link {
        source: 0,
        target: 0,
        delay: Delay::Milliseconds(500)
    });

    app.run();
}


