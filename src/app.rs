use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use eframe::egui;
use crossbeam_channel::Sender;

use crate::sound::*;

pub enum Message {
    PlaySound(SoundID),
    LoadSoundData(SoundID)
}

pub struct App {
    sender: Sender<Message>,
    shared: Arc<RwLock<SharedState>>,
    id_counter: usize
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
    pub fn new(sender: Sender<Message>, shared: Arc<RwLock<SharedState>>) -> Self {
        Self {
            sender,
            shared,
            id_counter: 0
        }
    }

    pub fn add_sound(&mut self, filename: String) -> SoundID {
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

    pub fn add_connection(&self, connection: Connection) {

    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Samples:");
            for (_id, sound) in self.shared.read().unwrap().sounds.iter() {
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