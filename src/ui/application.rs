use winit::{
    window::Window,
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::ui::primitive::{Renderer, RendererController};


pub trait ApplicationConfig {
    fn window_title(&self) -> &str;
}

pub trait Application: 'static + Sized {
    type Config: ApplicationConfig;

    fn init(config: Self::Config) -> Self;

    fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>);

    fn update(&mut self);

    fn draw(&self, controller: RendererController);

    fn handle_window_event(&mut self, event: &WindowEvent, window: &Window);

    fn run(config: Self::Config) {
        env_logger::init();
    
        let event_loop = EventLoop::new();
        
        let window = WindowBuilder::new()
            .with_title(config.window_title())
            .build(&event_loop).unwrap();
   
        let mut renderer = Renderer::init(&window);
        
        let mut app = Self::init(config);

        event_loop.run(move |event, _, control_flow| {
            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == window.id() => {
                    match event {
                        WindowEvent::CloseRequested => {
                            *control_flow = ControlFlow::Exit;
                        },
                        WindowEvent::Resized(physical_size) => {
                            renderer.resize(*physical_size);
                            app.resize(*physical_size);
                        },
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            renderer.resize(**new_inner_size);
                            app.resize(**new_inner_size);
                        },
                        _ => {},
                    };
                    app.handle_window_event(event, &window);
                },
                Event::RedrawRequested(window_id) if window_id == window.id() => {
                    let renderer_controller = renderer.controller();

                    app.draw(renderer_controller);
                    
                    renderer.render();
                },
                Event::MainEventsCleared => {
                    app.update();
                    window.request_redraw();
                },
                _ => {}
            }
        });
    }
}


