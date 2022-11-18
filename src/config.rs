use crate::{
    output::OutputConfig,
    sound::Sound,
    application::Float
};


#[derive(Default)]
pub struct Config {
    pub output: OutputConfig,
    pub sounds: Vec<Sound<Float>>
}