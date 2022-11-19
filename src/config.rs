use crate::sound::{Sound, OutputConfig, Float};


#[derive(Default)]
pub struct InstrumentConfig {
    pub output: OutputConfig,
    pub sounds: Vec<Sound<Float>>
}