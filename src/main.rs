#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::thread;
use std::sync::{Arc, RwLock};

use crossbeam_channel::unbounded;

mod app;
mod playback;
mod sound;
use app::{App, SharedState};
use playback::start_playback;


fn main() {
    // Crossbeam channel from main thread to playback thread
    let (sender, receiver) = unbounded();

    // Build app with shared state
    let shared = Arc::new(RwLock::new(SharedState::default()));
    let mut app = App::new(sender, shared.clone());

    // Start playback thread
    thread::spawn(move || { start_playback(receiver, shared.clone()) });
    
    // Load test sounds
    let sound_filenames = vec![
        "samples/kick.wav",
        "samples/snare.wav",
        "samples/hihat.wav"
    ];

    for filename in sound_filenames {
        app.add_sound(filename.to_string());
    }

    // Start GUI in main thread
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "State Machine",
        options,
        Box::new(|_cc| Box::new(app)),
    );
}


