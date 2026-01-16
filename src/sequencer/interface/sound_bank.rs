use crate::{
    sound::{SoundBankController, Float},
    ui::{primitive::{Drawable, Draw}, input::{InputHandler, Input}},
    instrument::{InstrumentState, Instrument}
};


pub struct SoundBankInterface {
    controller: SoundBankController<Float>
}

impl SoundBankInterface {
    pub fn new(controller: SoundBankController<Float>) -> SoundBankInterface {
        SoundBankInterface {
            controller
        }
    }
}

impl Drawable for SoundBankInterface {
    fn draw(&self, draw: &mut Draw) {
        todo!()
    }
}

impl InputHandler<Instrument> for SoundBankInterface {
    fn handle(&mut self, input: Input, state: InstrumentState) -> InstrumentState {
        todo!()
    }
}