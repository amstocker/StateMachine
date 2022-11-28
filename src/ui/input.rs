use winit::{dpi::{PhysicalPosition, PhysicalSize}, event::WindowEvent, window::Window};

use crate::ui::{Transform, Transformable, Position};
use crate::ui::application::Application;

use super::ApplyTransform;


pub trait InputHandler<A> where A: Application {
    fn handle(&mut self, input: Input, state: A::State) -> A::State;
}

pub struct Input<'a> {
    pub mouse_position: Position,
    pub input_type: InputType,
    pub event: &'a WindowEvent<'a>,
    pub window: &'a Window
}

impl<'a> Input<'a> {
    pub fn transform_and_defer_to<
        A: Application,
        T: InputHandler<A> + Transformable
    >(mut self, handler: &mut T, state: A::State) -> A::State {
        let transform = handler.transform();
        self.mouse_position = self.mouse_position.apply(
            transform.inverse()
        );
        handler.handle(self, state)
    }
}

pub enum InputType {
    MouseDown,
    MouseUp,
    MouseMove {
        moved_since_last_down: bool
    }
}


#[derive(Debug, Default)]
pub struct Mouse {
    pub position: MousePosition,
    pub position_delta: (f32, f32)
}

#[derive(Debug, Default, Clone, Copy)]
pub struct MousePosition {
    pub x: f32,
    pub y: f32,
}

impl MousePosition {
    pub fn delta(&self, last: MousePosition) -> (f32, f32) {
        (
            self.x - last.x,
            self.y - last.y
        )
    }

    pub fn from_physical(position: &PhysicalPosition<f64>, size: PhysicalSize<u32>) -> MousePosition {
        MousePosition {
            x: position.x as f32 / size.width as f32,
            y: position.y as f32 / size.height as f32
        }
    }

    pub fn transform(&self, transform: Transform) -> MousePosition {
        MousePosition { 
            x: transform.scale.0 * self.x + transform.translate.0,
            y: transform.scale.1 * self.y + transform.translate.1
        }
    }
}
