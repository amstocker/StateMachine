use wgpu::Color;
use winit::{
    window::Window,
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::ui::Transformable;
use crate::ui::Position;
use crate::ui::primitive::{Renderer, Drawable};
use crate::ui::input::{Input, InputType, InputHandler};


pub struct Style {
    pub clear_color: Color
}

pub trait ApplicationConfig<T> where T: Application {
    fn window_title(&self) -> &str;
    fn init_state(&self) -> Option<T::State>;
    fn style(&self) -> Style;
}

pub trait Application: 'static + Sized + Drawable + InputHandler<Self> {
    type Config: ApplicationConfig<Self>;
    type State: Default + Clone + Copy;

    fn init(config: Self::Config) -> Self;

    fn handle_resize(&mut self, size: winit::dpi::PhysicalSize<u32>);
    
    fn update(&mut self, state: Self::State) -> Self::State;

    fn run(config: Self::Config) {
        env_logger::init();
    
        let event_loop = EventLoop::new();
        
        let window = WindowBuilder::new()
            .with_title(config.window_title())
            .build(&event_loop).unwrap();
   
        let mut renderer = Renderer::init(&window, config.style());
        
        let mut app_state = if let Some(state) = config.init_state() {
            state
        } else {
            Self::State::default()
        };

        let mut app = Self::init(config);

        let mut mouse_position = Position::default();

        event_loop.run(move |event, _, control_flow| {
            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == window.id() => {
                    match event {
                        WindowEvent::CloseRequested => {
                            // app.handle_exit();
                            *control_flow = ControlFlow::Exit;
                        },
                        WindowEvent::Resized(physical_size) => {
                            renderer.resize(*physical_size);
                            app.handle_resize(*physical_size);
                        },
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            renderer.resize(**new_inner_size);
                            app.handle_resize(**new_inner_size);
                        },
                        _ => {}
                    }

                    // need to update mouse position and stuff here
                    let input = Input {
                        mouse_position,
                        input_type: InputType::MouseDown,
                        event,
                        window: &window
                    };
                    app_state = app.handle(input, app_state);
                },
                Event::RedrawRequested(window_id) if window_id == window.id() => {
                    let mut renderer_controller = renderer.controller();

                    app.draw(&mut renderer_controller);
                    
                    renderer.render();
                },
                Event::MainEventsCleared => {
                    app_state = app.update(app_state);
                    window.request_redraw();
                },
                _ => {}
            }
        });
    }
}


