use crate::{
    ui::{primitive::{Drawable, Draw}, input::{InputHandler, Input}},
    app::{AppState, App}
};


pub struct ControlLoopInterface {

}

impl Drawable for ControlLoopInterface {
    fn draw(&self, draw: &mut Draw) {
        todo!()
    }
}

impl InputHandler<App> for ControlLoopInterface {
    fn handle(&mut self, input: Input, state: AppState) -> AppState {
        todo!()
    }
}
