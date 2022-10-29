use std::collections::HashMap;
use std::sync::Arc;
use std::thread;

use crossbeam_channel::{unbounded, Sender, Receiver};
use parking_lot::RwLock;
use eframe::egui;

use crate::sound::*;
use crate::playback::*;

pub enum Message {
    PlaySound(SoundID),
    LoadSoundData(SoundID),
    //DeleteSoundData(SoundID)
}

pub struct App {
    pub sender: Sender<Message>,
    pub receiver: Receiver<Message>,
    pub trigger_map: HashMap<SoundID, Vec<TriggerInfo>>,
    pub shared: SharedState,
    id_counter: usize,
    pub queue_for_remove: Vec<SoundID>
}

#[derive(Clone)]
pub struct SharedState {
    pub sounds: Arc<RwLock<HashMap<SoundID, Sound>>>,
}

impl Default for SharedState {
    fn default() -> Self {
        Self {
            sounds: Arc::new(RwLock::new(HashMap::new()))
        }
    }
}

impl App {
    pub fn new() -> Self {
        let (sender, receiver) = unbounded();
        Self {
            sender,
            receiver,
            trigger_map: HashMap::new(),
            shared: SharedState::default(),
            id_counter: 0,
            queue_for_remove: Vec::new()
        }
    }

    pub fn run(self) {
        let receiver = self.receiver.clone();
        let shared = self.shared.clone();
        let trigger_map = self.trigger_map.clone();
        thread::spawn(move || { start_playback(receiver, trigger_map, shared) });

        let options = eframe::NativeOptions {
            // Hide the OS-specific "chrome" around the window:
            decorated: false,
            // To have rounded corners we need transparency:
            //transparent: true,
            min_window_size: Some(egui::vec2(320.0, 100.0)),
            ..Default::default()
        };
        eframe::run_native(
            "State Machine",
            options,
            Box::new(|_cc| Box::new(self)),
        );
    }

    pub fn add_sound(&mut self, filename: String) -> SoundID {
        // TODO: Use Pathbuf instead of filename, then display just the filename
        // use path for path, filename for just the filename!
        let id = self.id_counter;
        self.shared.sounds.write().insert(
            id,
            Sound {
                id,
                filename,
            }
        );
        self.sender.send(Message::LoadSoundData(id)).unwrap();
        self.id_counter += 1;
        id
    }

    pub fn add_trigger(&mut self, id: SoundID, trigger: TriggerInfo) {
        if let Some(triggers) = self.trigger_map.get_mut(&id) {
            triggers.push(trigger);
        }
    }
}


