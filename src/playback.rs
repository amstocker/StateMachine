use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::thread;

use crossbeam_channel::Receiver;
use rodio::{OutputStream, source::Source};

use crate::app::State;
use crate::sound::SoundID;


pub enum Message {
    Play(SoundID),
    Pause,
    Stop
}

pub enum PlaybackState {
    Playing,
    Paused,
    Stopped
}

pub enum Delay {
    Milliseconds(u32),
    Tempo {
        count: u32,
        division: u32,
        swing: f32
    }
}

pub struct Link {
    source: SoundID,
    target: SoundID,
    delay: Delay
}

pub struct Graph {
    pub link_map: HashMap<SoundID, Vec<Link>>
}

impl Graph {
    pub fn new() -> Self {
        Self {
            link_map: HashMap::new()
        }
    }

    pub fn add_link(&mut self, link: Link) {
        if let Some(links) = self.link_map.get_mut(&link.source) {
            links.push(link);
        } else {
            let source = link.source;
            let mut links = Vec::new();
            links.push(link);
            self.link_map.insert(source, links);
        }
    }
}

enum EventType {
    Trigger(SoundID),
    Tick
}

struct Event {
    event_type: EventType,
    time_until: usize
}


pub struct Engine {
    receiver: Receiver<Message>,
    app_state: Arc<State>,
    playback_state: PlaybackState,
}

impl Engine {
    pub fn new(app_state: Arc<State>, receiver: Receiver<Message>) -> Self {
        Self {
            receiver,
            app_state,
            playback_state: PlaybackState::Stopped
        }
    }

    pub fn run(mut self) {
        thread::spawn(move || {
            self.start_playback();
        });
    }

    pub fn start_playback(&mut self) {
        let (stream, stream_handle) = OutputStream::try_default().unwrap();

        let mut queue: HashMap<SoundID, VecDeque<Event>> = HashMap::new();
    
        self.playback_state = PlaybackState::Paused;
        loop {
            if let Ok(msg) = self.receiver.recv() {
                use Message::*;
                match msg {
                    Play(id) => {
                        loop {

                        }
                    },
                    Pause => {
                        // already paused
                    }
                    Stop => {
                        break;
                    }
                }
            }
        }
    }
}
