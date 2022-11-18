use bevy::prelude::*;

use crate::{fonts, sequencer::{Sequencer, SequencerController}, config::Config, output::{Output, OutputConfig}};


pub type Float = f32;

pub fn do_something_with_sequencer(
    time: Res<Time>,
    mut sequencer_controller: NonSendMut<SequencerController>
) {
    sequencer_controller.play_sound_once(0);
}

pub fn start_sequencer(world: &mut World) {
    let output_config = OutputConfig::default();

    let (sequencer_controller, sequencer) = Sequencer::new();

    let mut output = Output::new(output_config);
    output.start(sequencer);

    world.insert_non_send_resource(output);
    world.insert_non_send_resource(sequencer_controller);
}