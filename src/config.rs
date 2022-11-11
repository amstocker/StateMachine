use crate::output::OutputConfig;

pub struct Config {
    pub output: OutputConfig
}

impl Default for Config {
    fn default() -> Self {
        Self {
            output: Default::default()
        }
    }
}