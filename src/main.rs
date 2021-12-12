use std::iter;

use wgpu::RenderPipeline;
use winit::{window::{Window, WindowBuilder}, event::{WindowEvent, Event, KeyboardInput, VirtualKeyCode, ElementState}, event_loop::{EventLoop, ControlFlow}};

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline
}

impl State {
    async fn new(window: &Window) -> Self {

        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe {instance.create_surface(window)};
        let _adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {  
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },).await.unwrap();
        
        let adapter = instance 
            .enumerate_adapters(wgpu::Backends::all())
            .filter(|adapter|{
            surface.get_preferred_format(&adapter).is_some()
            }).next().unwrap();

       let (device, queue) = adapter.request_device(
           &wgpu::DeviceDescriptor {
               features: wgpu::Features::empty(),
               limits: wgpu::Limits::default(), 
               label: None,
           }, None,
       ).await.unwrap();

       let config = wgpu::SurfaceConfiguration {
           usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
           format: surface.get_preferred_format(&adapter).unwrap(),
           width: size.width,
           height: size.height,
           present_mode: wgpu::PresentMode::Fifo,
       };
       surface.configure(&device, &config);

       let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
           label: Some("Shader"),
           source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
       });

       let render_pipeline_layout = 
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[]
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                
                polygon_mode: wgpu::PolygonMode::Fill,

                clamp_depth: false,

                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        });

       Self {
           surface,
           device,
           queue,
           config,
           size,
           render_pipeline,
       }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>){
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {
        // nothing for now
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 1.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
               depth_stencil_attachment: None, 
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.draw(0..3, 0..1);
        }
        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    window.set_title("Hello World");
    let mut state: State = pollster::block_on(State::new(&window));

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested 
                        | WindowEvent::KeyboardInput {
                            input: 
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged {new_inner_size, ..} => {
                            state.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(_) => {
                state.update();
                match state.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    Err(e) => eprintln!("{:?}", e),         
            }
        }
        Event::RedrawEventsCleared => {
            window.request_redraw();
        }
        _ => {}
    }
    });

}