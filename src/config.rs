use crate::sound::{Sound, OutputConfig, Float};


#[derive(Default)]
pub struct Config {
    pub output: OutputConfig,
    pub sounds: Vec<Sound<Float>>
}