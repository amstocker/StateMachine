use winit::{dpi::{PhysicalPosition, PhysicalSize}, event::WindowEvent, window::Window};

use crate::ui::{Transform, Transformable, Position};
use crate::ui::application::Application;


pub trait InputHandler<A> where A: Application {
    fn handle(&mut self, input: Input<A>) -> A::State;
}

pub struct Input<'a, A> where A: Application {
    pub mouse_position: Position,
    pub input_type: InputType,
    pub event: &'a WindowEvent<'a>,
    pub window: &'a Window,
    pub state: A::State
}

pub enum InputType {
    MouseDown,
    MouseUp,
    MouseMove {
        moved_since_last_down: bool
    }
}

impl<'a, A> Input<'a, A> where A: Application {
    pub fn defer_to<T: Transformable + InputHandler<A>>(self, thing: &mut T) -> A::State {
        // transform mouse position
        thing.handle(self)
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
