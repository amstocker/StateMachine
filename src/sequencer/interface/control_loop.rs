use crate::{
    ui::{primitive::{Drawable, Draw}, input::{InputHandler, Input}},
    instrument::{InstrumentState, Instrument}
};


pub struct ControlLoopInterface {

}

impl Drawable for ControlLoopInterface {
    fn draw(&self, draw: &mut Draw) {
        todo!()
    }
}

impl InputHandler<Instrument> for ControlLoopInterface {
    fn handle(&mut self, input: Input, state: InstrumentState) -> InstrumentState {
        todo!()
    }
}