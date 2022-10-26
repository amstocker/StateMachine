use std::sync::Arc;
use std::collections::HashMap;

use crossbeam_channel::Receiver;
use parking_lot::RwLock;
use rodio::{OutputStream, source::Source};

use crate::app::{Message, SharedState};
use crate::sound::{SoundID, SoundData};


// probably want to create some kind of AudioEngine struct with start method?

pub fn start_playback(receiver: Receiver<Message>, shared: Arc<RwLock<SharedState>>) {
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
            match msg {
                Message::PlaySound(id) => {
                    if let Some(sound_data) = sound_data_map.get(&id) {
                        stream_handle.play_raw(sound_data.decoder().convert_samples()).unwrap();
                    } else {
                        panic!("Invalid ID");
                    }
                },
                Message::LoadSoundData(id) => {
                    if let Some(sound) = shared.read().sounds.get(&id) {
                        if let Ok(sound_data) = SoundData::load(&sound.filename) {
                            sound_data_map.insert(id, sound_data);
                        }
                    }
                }
            }
        }
    }   
    
}