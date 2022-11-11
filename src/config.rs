use crate::{
    output::OutputConfig,
    sound::Sound,
    application::Float
};


#[derive(Default)]
pub struct Config {
    pub output: OutputConfig,
    pub init_sounds: Vec<Sound<Float>>
}