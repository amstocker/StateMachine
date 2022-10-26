use std::collections::HashMap;
use std::sync::Arc;
use std::thread;

use eframe::egui;
use crossbeam_channel::{unbounded, Sender, Receiver};
use parking_lot::RwLock;

use crate::sound::*;
use crate::playback::*;

pub enum Message {
    PlaySound(SoundID),
    LoadSoundData(SoundID)
}

pub struct App {
    sender: Sender<Message>,
    receiver: Receiver<Message>,
    shared: Arc<RwLock<SharedState>>,
    id_counter: usize,
    queue_for_remove: Vec<SoundID>
}

pub struct SharedState {
    pub sounds: HashMap<SoundID, Sound>,
}

impl Default for SharedState {
    fn default() -> Self {
        Self {
            sounds: HashMap::new()
        }
    }
}

impl App {
    pub fn new() -> Self {
        let (sender, receiver) = unbounded();
        Self {
            sender,
            receiver,
            shared: Arc::new(RwLock::new(SharedState::default())),
            id_counter: 0,
            queue_for_remove: Vec::new()
        }
    }

    pub fn run(self) {
        // Start playback thread
        let receiver = self.receiver.clone();
        let shared = self.shared.clone();
        thread::spawn(move || { start_playback(receiver, shared) });

        // Start GUI in main thread
        let options = eframe::NativeOptions {
            drag_and_drop_support: true,
            ..Default::default()
        };
        eframe::run_native(
            "State Machine",
            options,
            Box::new(|_cc| Box::new(self)),
        );
    }

    pub fn add_sound(&mut self, filename: String) -> SoundID {
        let id = self.id_counter;
        self.shared.write().sounds.insert(
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

    pub fn add_connection(&self, connection: Connection) {

    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            
            ui.heading("Samples:");

            for (id, sound) in self.shared.read().sounds.iter() {
                ui.horizontal(|ui| {
                    ui.monospace(&sound.filename);
                    if ui.button("Play").clicked() {
                        self.sender.send(Message::PlaySound(sound.id)).unwrap();
                    }
                    if ui.button("Remove").clicked() {
                        self.queue_for_remove.push(*id);
                    }
                });
            }
            while !self.queue_for_remove.is_empty() {
                if let Some(id) = self.queue_for_remove.pop() {
                    self.shared.write().sounds.remove(&id);
                }
            }


            if ui.button("Open file…").clicked() {
                if let Some(filename) = rfd::FileDialog::new().pick_file() {
                    self.add_sound(filename.display().to_string());
                }
            }
        });
    }
}
