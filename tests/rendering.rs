//! Explicit GPU validation; ordinary CI tests need neither a window nor a GPU.
use state_machine::ui::primitive::{Line, LineHandler, Quad, QuadHandler, Text, TextHandler};
use state_machine::ui::{Depth, Transform};
use wgpu::*;

#[test]
#[ignore = "requires a GPU adapter; run cargo test --test rendering -- --ignored"]
fn large_batches_render_and_discarded_batches_do_not_reappear() {
    let instance = Instance::new(Backends::all());
    let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions::default()))
        .expect("GPU adapter required for this explicit test");
    eprintln!("GPU test adapter: {:?}", adapter.get_info());
    let (device, queue) = pollster::block_on(adapter.request_device(
        &DeviceDescriptor {
            label: None,
            features: Features::empty(),
            limits: Limits::default(),
        },
        None,
    ))
    .unwrap();
    device.push_error_scope(ErrorFilter::Validation);
    let format = TextureFormat::Rgba8Unorm;
    let size = Extent3d {
        width: 64,
        height: 64,
        depth_or_array_layers: 1,
    };
    let texture = |format, sample_count, usage| {
        device.create_texture(&TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count,
            dimension: TextureDimension::D2,
            format,
            usage,
        })
    };
    let target = texture(
        format,
        1,
        TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
    );
    let target_view = target.create_view(&Default::default());
    let msaa =
        texture(format, 4, TextureUsages::RENDER_ATTACHMENT).create_view(&Default::default());
    let depth = texture(
        TextureFormat::Depth32Float,
        4,
        TextureUsages::RENDER_ATTACHMENT,
    )
    .create_view(&Default::default());
    let depth_state = DepthStencilState {
        format: TextureFormat::Depth32Float,
        depth_write_enabled: true,
        depth_compare: CompareFunction::Greater,
        stencil: StencilState::default(),
        bias: DepthBiasState::default(),
    };
    let msaa_state = MultisampleState {
        count: 4,
        ..Default::default()
    };
    let mut quads = QuadHandler::init(&device, format, depth_state.clone(), msaa_state);
    let mut lines = LineHandler::init(&device, format, depth_state.clone(), msaa_state);
    let mut text = TextHandler::init(
        &device,
        format,
        depth_state,
        msaa_state,
        winit::dpi::PhysicalSize::new(64, 64),
    );
    // Simulate text submitted for a frame that is subsequently skipped.
    text.write(
        Text {
            label: "STALE".into(),
            position: (0.0, 0.0),
            scale: 30.0,
            color: Color::WHITE,
            depth: Depth::Top,
        },
        Transform::identity(),
    );
    text.clear();
    for i in 0..513 {
        let visible = i == 512;
        quads.write(
            Quad {
                position: if visible { (0.1, 0.1) } else { (2.0, 2.0) },
                size: (0.25, 0.25),
                color: Color::RED,
                depth: Depth::Mid,
            },
            Transform::identity(),
        );
        lines.write(
            Line {
                from: if visible { (0.75, 0.1) } else { (2.0, 2.0) },
                to: if visible { (0.75, 0.9) } else { (3.0, 3.0) },
                color: Color::GREEN,
                depth: Depth::Mid,
            },
            Transform::identity(),
        );
    }
    let readback = device.create_buffer(&BufferDescriptor {
        label: None,
        size: 64 * 64 * 4,
        usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut belt = wgpu::util::StagingBelt::new(1024);
    for frame in 0..2 {
        quads.prepare(&device, &queue).unwrap();
        lines.prepare(&device, &queue).unwrap();
        let mut encoder = device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &msaa,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &depth,
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(0.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });
            quads.render(&mut pass);
            lines.render(&mut pass);
        }
        text.render(&device, &mut belt, &mut encoder, &msaa, &depth, 64, 64);
        drop(encoder.begin_render_pass(&RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &msaa,
                resolve_target: Some(&target_view),
                ops: Operations {
                    load: LoadOp::Load,
                    store: false,
                },
            })],
            depth_stencil_attachment: None,
        }));
        encoder.copy_texture_to_buffer(
            target.as_image_copy(),
            ImageCopyBuffer {
                buffer: &readback,
                layout: ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(256.try_into().unwrap()),
                    rows_per_image: None,
                },
            },
            size,
        );
        belt.finish();
        queue.submit(Some(encoder.finish()));
        belt.recall();
        let (sender, receiver) = std::sync::mpsc::channel();
        readback
            .slice(..)
            .map_async(MapMode::Read, move |result| sender.send(result).unwrap());
        device.poll(Maintain::Wait);
        receiver.recv().unwrap().unwrap();
        {
            let pixels = readback.slice(..).get_mapped_range();
            let (pixels, _) = pixels.as_chunks::<4>();
            if frame == 0 {
                assert!(
                    pixels.iter().filter(|p| p[0] > 200 && p[1] == 0).count() > 100,
                    "the rectangle beyond the old capacity must render"
                );
                assert!(
                    pixels.iter().filter(|p| p[1] > 50 && p[0] == 0).count() > 20,
                    "the line beyond the old capacity must render"
                );
                assert!(
                    pixels.iter().all(|p| p[2] == 0),
                    "discarded white text must not render"
                );
            } else {
                assert!(
                    pixels.iter().all(|p| *p == [0, 0, 0, 255]),
                    "cleared shapes/text must not leak into the next frame"
                );
            }
        }
        readback.unmap();
        quads.clear();
        lines.clear();
        text.clear();
    }
    assert!(pollster::block_on(device.pop_error_scope()).is_none());
}
