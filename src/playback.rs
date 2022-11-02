use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::thread;
use std::cmp::min;

use crossbeam_channel::Receiver;
use rodio::OutputStreamHandle;
use rodio::{OutputStream, source::Source};

use crate::app::State;
use crate::sound::SoundID;


pub enum PlaybackControlMessage {
    Play(SoundID),
    Pause,
    Stop
}

pub enum PlaybackState {
    Playing,
    Paused,
    Stopped
}

// pub enum Delay {
//     Milliseconds(u32),
//     Tempo {
//         count: u32,
//         division: u32,
//         swing: f32
//     }
// }

#[derive(Clone, Copy)]
pub enum Delay {
    Milliseconds(u64)
}

pub fn delay_to_duration(delay: Delay) -> Duration {
    let Delay::Milliseconds(time) = delay;
    Duration::from_millis(time)
}

pub struct Link {
    pub source: SoundID,
    pub target: SoundID,
    pub delay: Delay
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

#[derive(Debug)]
struct TriggerEvent {
    sound_id: SoundID,
    created: Instant,
    time_until: Duration
}


pub struct Engine {
    app_state: Arc<State>,
    playback_control: Receiver<PlaybackControlMessage>,
    playback_state: PlaybackState,
    event_queue_map: HashMap<SoundID, VecDeque<TriggerEvent>>,
    events_to_queue: Vec<TriggerEvent>,
    sounds_to_play: Vec<SoundID>,
    output_stream_handle: Option<OutputStreamHandle>,
    tick: Instant,
    spin_sleeper: spin_sleep::SpinSleeper
}

impl Engine {
    pub fn new(app_state: Arc<State>, playback_control: Receiver<PlaybackControlMessage>) -> Self {
        Self {
            app_state,
            playback_control,
            playback_state: PlaybackState::Stopped,
            event_queue_map: HashMap::new(),
            events_to_queue: Vec::new(),
            sounds_to_play: Vec::new(),
            output_stream_handle: None,
            tick: Instant::now(),
            spin_sleeper: spin_sleep::SpinSleeper::new(10_000_000)
        }
    }

    pub fn run(mut self) {
        thread::spawn(move || {
            self.playback_loop();
        });
    }

    pub fn clear_event_queues(&mut self) {
        for (_id, event_queue) in &mut self.event_queue_map {
            event_queue.clear();
        }
    }

    pub fn handle_event_queue(&mut self) {
        
        // Queue sounds to play
        let graph = &self.app_state.graph.read();
        let mut min_time_until = Duration::from_micros(1_000_000);
        for (id, event_queue) in &mut self.event_queue_map {
            while let Some(event) = event_queue.front() {
                let time_since_created = self.tick.duration_since(event.created);
                let time_until_trigger = event.time_until.saturating_sub(time_since_created);
                if time_until_trigger.is_zero() {
                    self.sounds_to_play.push(*id);
                    event_queue.pop_front();
                } else {
                    min_time_until = min(min_time_until, time_until_trigger);
                    break;
                }
            }
        }
        drop(graph);

        let sounds = &self.app_state.sounds.read();
        if let Some(stream_handle) = &self.output_stream_handle {
            for id in &self.sounds_to_play {
                let sound_data = &sounds.get(&id).unwrap().data;
                stream_handle.play_raw(sound_data.decoder().convert_samples()).unwrap();
            }
        }
        drop(sounds);

        for id in self.sounds_to_play.drain(..) {
            if let Some(links) = graph.link_map.get(&id) {
                for link in links {
                    let time_until = delay_to_duration(link.delay);
                    min_time_until = min(min_time_until, time_until);
                    self.events_to_queue.push(TriggerEvent { 
                        sound_id: link.target,
                        created: self.tick.clone(),
                        time_until
                    });
                }
            }
        }

        for event in self.events_to_queue.drain(..) {
            if let Some(event_queue) = self.event_queue_map.get_mut(&event.sound_id) {
                event_queue.push_back(event);
            } else {
                let mut event_queue = VecDeque::new();
                let id = event.sound_id;
                event_queue.push_back(event);
                self.event_queue_map.insert(id, event_queue);
            }
        }

        let dt = min_time_until.saturating_sub(self.tick.elapsed());
        thread::sleep(dt);
    }

    pub fn playback_loop(&mut self) {
        use PlaybackState::*;
        use PlaybackControlMessage::*;

        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        self.output_stream_handle = Some(stream_handle);

        self.playback_state = Paused;
        loop {
            if let Ok(msg) = self.playback_control.recv() {
                match msg {
                    Play(id) => {
                        self.playback_state = Playing;
                        let mut event_queue = VecDeque::new();
                        event_queue.push_back(TriggerEvent {
                            sound_id: id,
                            created: Instant::now(),
                            time_until: Duration::from_millis(0)
                        });
                        self.event_queue_map.insert(id, event_queue);
                        loop {
                            self.handle_event_queue();
                            println!("elapsed since last tick: {:?}", self.tick.elapsed());
                            self.tick = Instant::now();
                            if let Ok(Pause) = self.playback_control.try_recv() {
                                self.clear_event_queues();
                                self.playback_state = Paused;
                                break;
                            }
                        }
                    },
                    Pause => {
                        // already paused
                    }
                    Stop => {
                        self.playback_state = Stopped;
                        break;
                    }
                }
            }
        }
    }
}
