//! Window/surface smoke test. Opens briefly, renders >128 of each shape, exercises
//! zero-size suspension and resize, and exits. Does not open an audio device.
use state_machine::ui::primitive::{Line, Quad, Renderer, Text};
use state_machine::ui::{Depth, Style};
use wgpu::Color;
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("StateMachine rendering smoke test")
        .with_inner_size(PhysicalSize::new(800, 600))
        .build(&event_loop)
        .unwrap();
    let mut renderer = Renderer::init(
        &window,
        Style {
            clear_color: Color::BLACK,
        },
    );
    let mut frames = 0;
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => *control_flow = ControlFlow::Exit,
            Event::WindowEvent { event: WindowEvent::Resized(size), .. } => renderer.resize(size),
            Event::MainEventsCleared => window.request_redraw(),
            Event::RedrawRequested(_) => {
                let mut draw = renderer.controller();
                for i in 0..1024 {
                    let x = (i % 32) as f32 / 32.0;
                    let y = (i / 32) as f32 / 32.0;
                    draw.quad(Quad { position: (x, y), size: (0.02, 0.02), color: Color::RED, depth: Depth::Mid });
                    draw.line(Line { from: (x, y), to: (x + 0.03, y + 0.03), color: Color::GREEN, depth: Depth::Front });
                }
                draw.text(Text { label: format!("Frame {frames}: 1024 quads + 1024 lines"), position: (0.0, 0.0),
                    scale: 24.0, color: Color::WHITE, depth: Depth::Top });
                if frames == 10 {
                    renderer.resize(PhysicalSize::new(0, 0));
                    // Include drawing while suspended, including a text queue.
                    renderer.controller().text(Text { label: "discard me".into(), position: (0.0, 0.0),
                        scale: 40.0, color: Color::WHITE, depth: Depth::Top });
                    renderer.render().unwrap();
                    renderer.resize(window.inner_size());
                    window.set_inner_size(PhysicalSize::new(640, 480));
                } else {
                    renderer.render().unwrap();
                }
                frames += 1;
                if frames == 120 {
                    eprintln!("Rendering smoke test completed: 120 frames, resize and zero-size suspension.");
                    *control_flow = ControlFlow::Exit;
                }
            }
            _ => {}
        }
    });
}
