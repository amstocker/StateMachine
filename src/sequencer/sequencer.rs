use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, AtomicBool};
use std::sync::atomic::Ordering::SeqCst;

use crate::application::Float;
use crate::sound::{SoundBank, MAX_SOUNDS};
use crate::output::{StereoFrame, StereoFrameGenerator};


pub const MAX_NODES: usize = 8;

pub const INPUT_TRIGGERS_PER_NODE: usize = 4;
pub const OUTPUT_TRIGGERS_PER_NODE: usize = 4;

#[derive(Default)]
pub struct Nodes([Node; MAX_NODES]);

#[derive(Default)]
pub struct InputTriggers([TriggerInput; INPUT_TRIGGERS_PER_NODE * MAX_NODES]);

#[derive(Default)]
pub struct OutputTriggers([TriggerOutput; OUTPUT_TRIGGERS_PER_NODE * MAX_NODES]);

impl Nodes {
    #[inline]
    pub fn get(&self, node_index: usize) -> &Node {
        &self.0[node_index]
    }
}

impl InputTriggers {
    #[inline]
    pub fn get(&self, node_index: usize, input_number: usize) -> &TriggerInput {
        &self.0[(node_index * INPUT_TRIGGERS_PER_NODE) + input_number]
    }

    #[inline]
    fn get_cache(&self, node_index: usize, input_number: usize) -> TriggerInputCache {
        self.get(node_index, input_number).cache()
    }

    fn set(&self, node_index: usize, input_number: usize, cache: TriggerInputCache) {
        let trigger = self.get(node_index, input_number);
        trigger.frames_until.store(cache.frames_until, SeqCst);
        trigger.pending.store(cache.pending, SeqCst);
    }
}

impl OutputTriggers {
    #[inline]
    pub fn get(&self, node_index: usize, output_number: usize) -> &TriggerOutput {
        &self.0[(node_index * OUTPUT_TRIGGERS_PER_NODE) + output_number]
    }

    #[inline]
    fn get_cache(&self, node_index: usize, output_number: usize) -> TriggerOutputCache {
        self.get(node_index, output_number).cache()
    }

    fn set(&self, node_index: usize, output_number: usize, cache: TriggerOutputCache) {
        let trigger = self.get(node_index, output_number);
        trigger.target_index.store(cache.target_index, SeqCst);
        trigger.target_input_number.store(cache.target_input_number, SeqCst);
        trigger.frame_delay.store(cache.frame_delay, SeqCst);
        trigger.enabled.store(cache.enabled, SeqCst);
    }
}

#[derive(Default)]
pub struct Node {
    pub sound_index: AtomicUsize,
    pub is_playing: AtomicBool,
    pub current_frame_index: AtomicUsize,
    pub enabled: AtomicBool
}

#[derive(Default)]
struct Playhead {
    current_frame_index: AtomicUsize,
    is_playing: AtomicBool
}

#[derive(Default, Clone, Copy)]
struct NodeInternalMetadata {
    triggered_this_frame: bool,
    triggered_last_frame: bool
}

#[derive(Default)]
pub struct TriggerInput {
    pub frames_until: AtomicUsize,
    pub pending: AtomicBool,
}

pub struct TriggerInputCache {
    frames_until: usize,
    pending: bool
}

impl TriggerInput {
    fn cache(&self) -> TriggerInputCache {
        TriggerInputCache {
            frames_until: self.frames_until.load(SeqCst),
            pending: self.pending.load(SeqCst)
        }
    }
}

#[derive(Default)]
pub struct TriggerOutput {
    pub target_index: AtomicUsize,
    pub target_input_number: AtomicUsize,
    pub frame_delay: AtomicUsize,
    pub enabled: AtomicBool
}

struct TriggerOutputCache {
    target_index: usize,
    target_input_number: usize,
    frame_delay: usize,
    enabled: bool
}

impl TriggerOutput {
    fn cache(&self) -> TriggerOutputCache {
        TriggerOutputCache { 
            target_index: self.target_index.load(SeqCst),
            target_input_number: self.target_input_number.load(SeqCst),
            frame_delay: self.frame_delay.load(SeqCst),
            enabled: self.enabled.load(SeqCst)
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SequencerControlMessage {
    EnableSound(usize),
    DisableSound(usize),
    PlaySoundOnce(usize),
    IncrSoundIndex(usize),
    DecrSoundIndex(usize)
}

#[derive(Default, Clone)]
pub struct SequencerParameters {
    pub nodes: Arc<Nodes>,
    pub input_triggers: Arc<InputTriggers>,
    pub output_triggers: Arc<OutputTriggers>,
}

impl SequencerParameters {
    pub fn handle_message(&self, message: SequencerControlMessage) {
        use SequencerControlMessage::*;
        match message {
            EnableSound(index) => {
                let node = self.nodes.get(index);
                node.enabled.store(true, SeqCst);
            },
            DisableSound(index) => {
                let node = self.nodes.get(index);
                node.enabled.store(false, SeqCst);
            },
            PlaySoundOnce(index) => {
                let node = self.nodes.get(index);
                node.current_frame_index.store(0, SeqCst);
                node.is_playing.store(true, SeqCst);
            },
            IncrSoundIndex(index) => {
                let node = self.nodes.get(index);
                let sound_index = node.sound_index.load(SeqCst);
                if sound_index < MAX_SOUNDS {
                    node.sound_index.fetch_add(1, SeqCst);
                }
            },
            DecrSoundIndex(index) => {
                let node = self.nodes.get(index);
                let sound_index = node.sound_index.load(SeqCst);
                if sound_index > 0 {
                    node.sound_index.fetch_sub(1, SeqCst);
                }
            }
        }
    }
}

pub struct Sequencer {
    sound_bank: SoundBank<Float>,
    nodes: Arc<Nodes>,
    input_triggers: Arc<InputTriggers>,
    output_triggers: Arc<OutputTriggers>,
    nodes_internal: [NodeInternalMetadata; MAX_NODES],
    frames_processed: u64
}

impl Sequencer {
    pub fn new(sound_bank: SoundBank<Float>) -> (SequencerParameters, Sequencer) {
        let sequencer_parameters = SequencerParameters::default();
        let sequencer = Sequencer {
            sound_bank,
            nodes: sequencer_parameters.nodes.clone(),
            input_triggers: sequencer_parameters.input_triggers.clone(),
            output_triggers: sequencer_parameters.output_triggers.clone(),
            nodes_internal: Default::default(),
            frames_processed: 0
        };

        (sequencer_parameters, sequencer)
    }

    pub fn update_single_frame(&mut self) {
        for i in 0..MAX_NODES {
            let node = self.nodes.get(i);
            let mut node_internal = &mut self.nodes_internal[i];
            if node.enabled.load(SeqCst) && node_internal.triggered_last_frame {
                for j in 0..OUTPUT_TRIGGERS_PER_NODE {
                    let trigger = &self.output_triggers.get_cache(i, j);
                    if trigger.enabled {
                        self.input_triggers.set(
                            trigger.target_index,
                            trigger.target_input_number,
                            TriggerInputCache { 
                                frames_until: trigger.frame_delay - 1,
                                pending: true
                            }
                        );
                    }
                }
                node_internal.triggered_last_frame = false;
            }
        }
        for i in 0..MAX_NODES {
            let node = self.nodes.get(i);
            let mut node_internal = &mut self.nodes_internal[i];
            if node.enabled.load(SeqCst) {
                for j in 0..INPUT_TRIGGERS_PER_NODE {
                    let trigger = &self.input_triggers.get(i, j);
                    if trigger.pending.load(SeqCst) {
                        if trigger.frames_until.load(SeqCst) == 0 {
                            node_internal.triggered_this_frame = true;
                            trigger.pending.store(false, SeqCst);
                        } else {
                            trigger.frames_until.fetch_sub(1, SeqCst);
                        }
                    }
                }
            }
        }
    }

    pub fn output_single_frame(&mut self) -> StereoFrame<Float> {
        self.sound_bank.update();
        self.update_single_frame();
        
        let mut out_frame = StereoFrame::zero();
        for i in 0..MAX_NODES {
            let node = self.nodes.get(i);
            let mut node_internal = &mut self.nodes_internal[i];
            if node.enabled.load(SeqCst) {
                if node_internal.triggered_this_frame {
                    node.is_playing.store(true, SeqCst);
                    node.current_frame_index.store(0, SeqCst);
                    node_internal.triggered_this_frame = false;
                }
                if node.is_playing.load(SeqCst) {
                    let sound_id = node.sound_index.load(SeqCst);
                    let frame_index = node.current_frame_index.load(SeqCst);
                    if let Some(frame) = self.sound_bank.get_frame(sound_id, frame_index) {
                        out_frame += frame;
                        node.current_frame_index.fetch_add(1, SeqCst);
                    } else {
                        node.is_playing.store(false, SeqCst);
                    }
                }
            }
        }
        self.frames_processed += 1;
        out_frame
    }
}

impl StereoFrameGenerator<Float> for Sequencer {
    fn next_frame(&mut self) -> StereoFrame<Float> {
        self.output_single_frame()
    }
}
