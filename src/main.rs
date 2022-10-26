#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
mod app;
mod playback;
mod sound;

use app::App;


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

    app.run();
}


