use std::collections::{HashMap, BinaryHeap};
use std::cmp::Ordering;
use std::time::{Duration, Instant};
use std::thread::sleep;

use crossbeam_channel::Receiver;
use rodio::{OutputStream, source::Source};

use crate::app::{Message, SharedState};
use crate::sound::{SoundID, SoundData, TriggerInfo};


// probably want to create some kind of AudioEngine struct with start method?

#[derive(Eq, PartialEq)]
enum EventType { // use trait instead?
    Trigger(SoundID),
    Tick
}

#[derive(PartialEq, Eq)]
struct Event {
    event_type: EventType,
    time_created: Instant,
    ms_until: usize
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> Ordering {
        let ordering = other.ms_until.cmp(&self.ms_until);

        use EventType::*;
        if let (Trigger(l_id), Trigger(r_id)) = (&self.event_type, &other.event_type) {
            if ordering.is_eq() {
                return r_id.cmp(&l_id);
            }
        }
        ordering
    }
}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}


pub fn start_playback(receiver: Receiver<Message>, routing: HashMap<SoundID, Vec<TriggerInfo>>, shared: SharedState) {
    // TICK = 1 ms (or e.g. 500 ms for 120 bpm)
    //      (collect some data on timing)
    // (1) use priority queue to play sounds with tick = 0
    // (2) queue next sounds based on routing configuration
    // (3) iterate over rest to decrease ticks for all queued sounds.
    // (4) use Instand::now() to sleep thread for time delta until next tick
    //      (or just sleep until next sample scheduled)

    /*
     *  Notes:
     *      - Adding a sound stops playback
     *      - All sounds are loaded BEFORE playback...
     *      - So app is created, loads sounds into hashmap, then this
     *        map is passed to the playback thread when playback starts.
     */
    
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    let mut sound_data_map: HashMap<SoundID, SoundData> = HashMap::new();


    let mut schedule: BinaryHeap<Event> = BinaryHeap::new();
    let mut new_schedule: BinaryHeap<Event> = BinaryHeap::new();

    let last = Instant::now();
    let now = Instant::now();

    let mut event = None;
    loop {
        std::mem::swap(&mut schedule, &mut new_schedule);

        event = schedule.pop();
        while event.is_some() {
            
            // handle event
            // else decrease time until and reinsert
            event = schedule.pop();
        }
    }
    // loop {
    //     if let Ok(msg) = receiver.recv() {
    //         match msg {
    //             Message::PlaySound(id) => {
    //                 if let Some(sound_data) = sound_data_map.get(&id) {
    //                     stream_handle.play_raw(sound_data.decoder().convert_samples()).unwrap();
    //                 } else {
    //                     panic!("Invalid ID");
    //                 }
    //             },
    //             Message::LoadSoundData(id) => {
    //                 if let Some(sound) = shared.sounds.read().get(&id) {
    //                     if let Ok(sound_data) = SoundData::load(&sound.filename) {
    //                         sound_data_map.insert(id, sound_data);
    //                     }
    //                 }
    //             }
    //         }
    //     }
    // }   
    
}