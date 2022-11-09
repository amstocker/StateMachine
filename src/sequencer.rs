use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, AtomicBool};
use std::sync::atomic::Ordering::SeqCst;

use crate::sound::{SoundBank};
use crate::output::{OutputSample, StereoFrame};


// Sound nodes are on a four-by-four grid
pub const GRID_SIZE: usize = 16;

pub const INPUT_TRIGGERS_PER_NODE: usize = 4;
pub const OUTPUT_TRIGGERS_PER_NODE: usize = 4;


pub type Grid = [Node; GRID_SIZE];
pub struct InputTriggers([TriggerInput; INPUT_TRIGGERS_PER_NODE * GRID_SIZE]);
pub struct OutputTriggers([TriggerOutput; OUTPUT_TRIGGERS_PER_NODE * GRID_SIZE]);

impl InputTriggers {
    #[inline]
    pub fn get(&self, node_index: usize, input_number: usize) -> &TriggerInput {
        &self.0[(node_index * INPUT_TRIGGERS_PER_NODE) + input_number]
    }

    #[inline]
    fn get_cache(&self, node_index: usize, input_number: usize) -> TriggerInputCache {
        self.get(node_index, input_number).cache()
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
}

#[derive(Debug)]
pub struct Node {
    pub sound_index: AtomicUsize,
    pub is_playing: AtomicBool,
    pub current_frame_index: AtomicUsize,
    pub enabled: AtomicBool
}

impl Node {
    pub fn new() -> Self {
        Self {
            sound_index: AtomicUsize::new(0),
            is_playing: AtomicBool::new(false),
            current_frame_index: AtomicUsize::new(0),
            enabled: AtomicBool::new(false)
        }
    }
}

#[derive(Clone, Copy)]
struct NodeInternal {
    triggered_this_frame: bool,
    triggered_last_frame: bool
}

#[derive(Debug)]
pub struct TriggerInput {
    pub frames_until: AtomicUsize,
    pub pending: AtomicBool,
}

pub struct TriggerInputCache {
    frames_until: usize,
    pending: bool
}

impl TriggerInput {
    pub fn new() -> Self {
        Self {
            frames_until: AtomicUsize::new(0),
            pending: AtomicBool::new(false),
        }
    }

    fn cache(&self) -> TriggerInputCache {
        TriggerInputCache {
            frames_until: self.frames_until.load(SeqCst),
            pending: self.pending.load(SeqCst)
        }
    }
}

#[derive(Debug)]
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
    pub fn new() -> Self {
        Self {
            target_index: AtomicUsize::new(0),
            target_input_number: AtomicUsize::new(0),
            frame_delay: AtomicUsize::new(0),
            enabled: AtomicBool::new(false)
        }
    }

    fn cache(&self) -> TriggerOutputCache {
        TriggerOutputCache { 
            target_index: self.target_index.load(SeqCst),
            target_input_number: self.target_input_number.load(SeqCst),
            frame_delay: self.frame_delay.load(SeqCst),
            enabled: self.enabled.load(SeqCst)
        }
    }
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
        let nodes: Arc<Grid> = Arc::new(
            (0..GRID_SIZE).map(|_| Node::new())
                .collect::<Vec<Node>>().try_into().unwrap()
        );
        let input_triggers: Arc<InputTriggers> = Arc::new(InputTriggers(
            (0..INPUT_TRIGGERS_PER_NODE * GRID_SIZE).map(|_| TriggerInput::new())
                .collect::<Vec<TriggerInput>>().try_into().unwrap()
        ));
        let output_triggers: Arc<OutputTriggers> = Arc::new(OutputTriggers(
            (0..OUTPUT_TRIGGERS_PER_NODE * GRID_SIZE).map(|_| TriggerOutput::new())
                .collect::<Vec<TriggerOutput>>().try_into().unwrap()
        ));

        let controller = SequencerController {
            nodes: nodes.clone(),
            input_triggers: input_triggers.clone(),
            output_triggers: output_triggers.clone()
        };

        let node_internal_init = NodeInternal {
            triggered_last_frame: false,
            triggered_this_frame: false
        };
        let sequencer = Sequencer {
            sound_bank,
            nodes,
            nodes_internal: [node_internal_init; GRID_SIZE],
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
                    let trigger = &self.output_triggers.get_cache(i, j);
                    if trigger.enabled {
                        let target = &self.input_triggers.get(trigger.target_index, trigger.target_input_number);
                        target.frames_until.store(trigger.frame_delay - 1, SeqCst);
                        target.pending.store(true, SeqCst);
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

    pub fn next_frame(&mut self) -> StereoFrame<S> {
        self.update_single_frame();
        self.output_single_frame()
    }

    fn split_borrow(&mut self) -> (&Arc<Grid>, &Arc<InputTriggers>, &Arc<OutputTriggers>) {
        (&mut self.nodes, &mut self.input_triggers, &mut self.output_triggers)
    }


}

impl<S> Iterator for Sequencer<S> where S: OutputSample {
    type Item = StereoFrame<S>;

    fn next(&mut self) -> Option<Self::Item> {
        self.update_single_frame();
        Some(self.output_single_frame())
    }
}