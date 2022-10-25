#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::fs::read;
use std::io::{BufReader, Cursor};
use std::thread;
use std::sync::{Arc, RwLock};
use std::collections::{BinaryHeap, HashMap};

use rodio::{Decoder, OutputStream, source::Source};
use eframe::egui;
use crossbeam_channel::{unbounded, Receiver, Sender};

mod sound_data;
use sound_data::SoundData;


fn main() {
    // Crossbeam channel from main thread to playback thread
    let (sender, receiver) = unbounded();

    // Build app with shared state
    let shared = Arc::new(RwLock::new(SharedState::default()));
    let mut app = App::new(sender, shared.clone());

    // Start playback thread
    thread::spawn(move || { playback(receiver, shared.clone()) });
    
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

enum Message {
    PlaySound(SoundID),
    LoadSoundData(SoundID)
}

type SoundID = usize;

struct Sound {
    id: SoundID,
    filename: String,
    connections: Vec<Connection>
}

enum Delay {
    Milliseconds(u32),
    Tempo {
        count: u32,
        division: u32,
        swing: f32
    }
}

struct Connection {
    target: SoundID,
    delay: Delay
}

fn playback(receiver: Receiver<Message>, shared: Arc<RwLock<SharedState>>) {
    // TICK = 1 ms (or e.g. 500 ms for 120 bpm)
    //      (collect some data on timing)
    // (1) use priority queue to play sounds with tick = 0
    // (2) queue next sounds based on routing configuration
    // (3) iterate over rest to decrease ticks for all queued sounds.
    // (4) use Instand::now() to sleep thread for time delta until next tick
    //      (or just sleep until next sample scheduled)
    
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    // let mut heap = BinaryHeap::new();
    let mut sound_data_map: HashMap<SoundID, SoundData> = HashMap::new();

    loop {
        if let Ok(msg) = receiver.recv() {
            use crate::Message::*;
            match msg {
                PlaySound(id) => {
                    if let Some(sound_data) = sound_data_map.get(&id) {
                        stream_handle.play_raw(sound_data.decoder().convert_samples()).unwrap();
                    } else {
                        panic!("Invalid ID");
                    }
                },
                LoadSoundData(id) => {
                    if let Some(sound) = shared.read().unwrap().sounds.get(&id) {
                        if let Ok(sound_data) = SoundData::load(&sound.filename) {
                            sound_data_map.insert(id, sound_data);
                        }
                    }
                }
            }
        }
    }   
    
}

#[derive(Clone)]
struct App {
    sender: Sender<Message>,
    shared: Arc<RwLock<SharedState>>,
    id_counter: usize
}

struct SharedState {
    sounds: HashMap<SoundID, Sound>,
}

impl Default for SharedState {
    fn default() -> Self {
        Self {
            sounds: HashMap::new()
        }
    }
}

impl App {
    fn new(sender: Sender<Message>, shared: Arc<RwLock<SharedState>>) -> Self {
        Self {
            sender,
            shared,
            id_counter: 0
        }
    }

    fn add_sound(&mut self, filename: String) -> SoundID {
        let id = self.id_counter;
        self.shared.write().unwrap().sounds.insert(
            id,
            Sound {
                id,
                filename,
                connections: Vec::new()
            }
        );
        self.sender.send(Message::LoadSoundData(id)).unwrap();
        self.id_counter += 1;
        id
    }

    fn add_connection(&self, connection: Connection) {

    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Samples:");
            for (id, sound) in self.shared.read().unwrap().sounds.iter() {
                ui.horizontal(|ui| {
                    ui.label(&sound.filename);
                    if ui.button("Play").clicked() {
                        self.sender.send(Message::PlaySound(sound.id)).unwrap();
                    }
                });
            }
        });
    }
}
