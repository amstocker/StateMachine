use std::collections::HashMap;
use std::sync::Arc;

use crossbeam_channel::{unbounded, Sender, Receiver};
use parking_lot::RwLock;

use crate::sound::*;
use crate::playback::*;
use crate::ui::UI;

pub struct App {
    pub state: Arc<State>,
    pub engine: Engine,
    pub ui: UI,
}

pub struct State {
    pub sounds: RwLock<HashMap<SoundID, Sound>>,
    pub graph: RwLock<Graph>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            sounds: RwLock::new(HashMap::new()),
            graph: RwLock::new(Graph::new()),
        }
    }
}

impl App {
    pub fn new() -> Self {
        let (sender, receiver) = unbounded();
        let state = Arc::new(State::default());
        Self {
            state: state.clone(),
            engine: Engine::new(state.clone(), receiver.clone()),
            ui: UI::new(state.clone(), sender.clone()),
        }
    }

    pub fn run(self) {
        // Playback engine runs in separate thread
        self.engine.run();
        // UI runs in main thread
        self.ui.run();
    }

    pub fn add_sound_to_state(app_state: Arc<State>, path: String) {
        let mut sounds = app_state.sounds.write();
        let sound = Sound::new(path);
        sounds.insert(sound.id, sound);
    }

    pub fn add_sound(&mut self, path: String) {
        let app_state = self.state.clone();
        Self::add_sound_to_state(app_state, path);
    }
}


