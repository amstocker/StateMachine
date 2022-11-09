use std::collections::VecDeque;
use std::mem::{MaybeUninit, self};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, AtomicBool};
use std::sync::atomic::Ordering::SeqCst;

use crate::sound::{SoundID, SoundBank};
use crate::output::{Frames, OutputSample, StereoFrame};


// Sound nodes are on a four-by-four grid
pub const GRID_SIZE: usize = 16;

pub const INPUT_TRIGGERS_PER_NODE: usize = 4;
pub const OUTPUT_TRIGGERS_PER_NODE: usize = 4;

pub type Grid = [Node; GRID_SIZE];
pub type InputTriggers = [TriggerInput; INPUT_TRIGGERS_PER_NODE * GRID_SIZE];
pub type OutputTriggers = [TriggerOutput; OUTPUT_TRIGGERS_PER_NODE * GRID_SIZE];

#[inline]
fn to_input_trigger_index(node_index: usize, input_number: usize) -> usize {
    (node_index * INPUT_TRIGGERS_PER_NODE) + input_number
}

#[inline]
fn to_output_trigger_index(node_index: usize, output_number: usize) -> usize {
    (node_index * OUTPUT_TRIGGERS_PER_NODE) + output_number
}


// UI thread: read & write
// Realtime thread: read only
pub struct TriggerOutput {
    pub target_index: AtomicUsize,
    pub target_input_number: AtomicUsize,
    pub frame_delay: AtomicUsize,
    pub enabled: AtomicBool
}

// UI thread: read only
// Realtime thread: read & write
pub struct TriggerInput {
    pub frames_until: AtomicUsize,
    pub pending: AtomicBool,
}

pub struct Node {
    pub sound_id: AtomicUsize,
    pub is_playing: AtomicBool,
    pub current_frame_index: AtomicUsize,
    pub enabled: AtomicBool
}

#[derive(Clone, Copy)]
struct NodeInternal {
    triggered_this_frame: bool,
    triggered_last_frame: bool
}

pub struct SequencerController {
    pub nodes: Arc<Grid>,
    pub input_triggers: Arc<InputTriggers>,
    pub output_triggers: Arc<OutputTriggers>,
}

pub struct Sequencer<S> where S: OutputSample {
    sound_bank: SoundBank<S>,
    nodes: Arc<Grid>,
    nodes_internal: [NodeInternal; GRID_SIZE],
    input_triggers: Arc<InputTriggers>,
    output_triggers: Arc<OutputTriggers>,
    frames_processed: u64
}

impl<S> Sequencer<S> where S: OutputSample {
    pub fn new() -> (SequencerController, Sequencer<S>) {
        Sequencer::new_with_sound_bank(SoundBank::new())
    }

    pub fn new_with_sound_bank(sound_bank: SoundBank<S>) -> (SequencerController, Sequencer<S>) {
        let nodes = Arc::new({
            let mut nodes_uninit: [MaybeUninit<Node>; GRID_SIZE] = unsafe {
                MaybeUninit::uninit().assume_init()
            };
            for elem in &mut nodes_uninit[..] {
                elem.write(Node {
                    sound_id: AtomicUsize::new(0),
                    is_playing: AtomicBool::new(false),
                    current_frame_index: AtomicUsize::new(0),
                    enabled: AtomicBool::new(false)
                });
            }
            unsafe {
                mem::transmute::<_, Grid>(nodes_uninit)
            }
        });
        let input_triggers = Arc::new({
            let mut triggers_uninit: [MaybeUninit<TriggerInput>; INPUT_TRIGGERS_PER_NODE * GRID_SIZE] = unsafe {
                MaybeUninit::uninit().assume_init()
            };
            for elem in &mut triggers_uninit[..] {
                elem.write(TriggerInput {
                    frames_until: AtomicUsize::new(0),
                    pending: AtomicBool::new(false),
                });
            }
            unsafe {
                mem::transmute::<_, InputTriggers>(triggers_uninit)
            }
        });
        let output_triggers = Arc::new({
            let mut triggers_uninit: [MaybeUninit<TriggerOutput>; OUTPUT_TRIGGERS_PER_NODE * GRID_SIZE] = unsafe {
                MaybeUninit::uninit().assume_init()
            };
            for elem in &mut triggers_uninit[..] {
                elem.write(TriggerOutput {
                    target_index: AtomicUsize::new(0),
                    target_input_number: AtomicUsize::new(0),
                    frame_delay: AtomicUsize::new(0),
                    enabled: AtomicBool::new(false)
                });
            }
            unsafe {
                mem::transmute::<_, OutputTriggers>(triggers_uninit)
            }
        });

        let controller = SequencerController {
            nodes: nodes.clone(),
            input_triggers: input_triggers.clone(),
            output_triggers: output_triggers.clone()
        };

        let sequencer = Sequencer {
            sound_bank,
            nodes,
            nodes_internal: [NodeInternal { triggered_last_frame: false, triggered_this_frame: false}; GRID_SIZE],
            input_triggers,
            output_triggers,
            frames_processed: 0
        };

        (controller, sequencer)
    }

    pub fn update_single_frame(&mut self) {
        for i in 0..GRID_SIZE {
            let node = &self.nodes[i];
            let mut node_internal = &mut self.nodes_internal[i];
            if node.enabled.load(SeqCst) && node_internal.triggered_last_frame {
                for j in 0..OUTPUT_TRIGGERS_PER_NODE {
                    let trigger = &self.output_triggers[to_output_trigger_index(i, j)];
                    if trigger.enabled.load(SeqCst) {
                        let target_index = trigger.target_index.load(SeqCst);
                        let target_input_number = trigger.target_input_number.load(SeqCst);
                        let frame_delay = trigger.frame_delay.load(SeqCst);

                        let target_input_trigger = &self.input_triggers[to_input_trigger_index(target_index, target_input_number)];
                        target_input_trigger.frames_until.store(frame_delay - 1, SeqCst);
                        target_input_trigger.pending.store(true, SeqCst);
                    }
                }
                node_internal.triggered_last_frame = false;
            }
        }
        for i in 0..GRID_SIZE {
            let node = &self.nodes[i];
            let mut node_internal = &mut self.nodes_internal[i];
            if node.enabled.load(SeqCst) {
                for j in 0..INPUT_TRIGGERS_PER_NODE {
                    let trigger = &self.input_triggers[to_input_trigger_index(i, j)];
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

    pub fn output_single_frame(&mut self) -> StereoFrame<S> {
        let mut out_frame = StereoFrame::zero();
        
        for i in 0..GRID_SIZE {
            let node = &self.nodes[i];
            let mut node_internal = &mut self.nodes_internal[i];
            if node.enabled.load(SeqCst) {
                if node_internal.triggered_this_frame {
                    node.is_playing.store(true, SeqCst);
                    node.current_frame_index.store(0, SeqCst);
                    node_internal.triggered_this_frame = false;
                }
                if node.is_playing.load(SeqCst) {
                    let sound_id = node.sound_id.load(SeqCst);
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

    pub fn next_frame(&mut self) -> StereoFrame<S> {
        self.update_single_frame();
        self.output_single_frame()
    }
}

impl<S> Iterator for Sequencer<S> where S: OutputSample {
    type Item = StereoFrame<S>;

    fn next(&mut self) -> Option<Self::Item> {
        self.update_single_frame();
        Some(self.output_single_frame())
    }
}