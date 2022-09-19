use std::iter;

use crate::math::UVec2;

use super::{
    gpu_interface::GPUInterface,
    math::{IVec2, Vertex},
};
use bytemuck::{Pod, Zeroable};
use wgpu::{util::DeviceExt, Buffer, SurfaceTexture};

// main.rs

use winit::{
    event::{Event, WindowEvent},
    window::Window,
};

pub struct Renderer {
    pub render_pipeline: wgpu::RenderPipeline,
    pub texture_bind_group_layout: wgpu::BindGroupLayout,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
    pub sampler: wgpu::Sampler,
    pub size: UVec2,
}

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-1.0, 1.0, 0.0],
        tex_coords: [0.0, 0.0],
    }, // A
    Vertex {
        position: [-1.0, -1.0, 0.0],
        tex_coords: [0.0, 1.0],
    }, // B
    Vertex {
        position: [1.0, -1.0, 0.0],
        tex_coords: [1.0, 1.0],
    }, // C
    Vertex {
        position: [1.0, 1.0, 0.0],
        tex_coords: [1.0, 0.0],
    }, // D
];

const INDICES: &[u16] = &[0, 1, 2, 2, 3, 0];

impl Renderer {
    // Creating some of the wgpu types requires async code
    pub fn new(gpu: &GPUInterface, size: UVec2, window: &Window) -> Self {
        let sampler = gpu.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders/render.wgsl").into()),
            });

        let texture_bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            // This should match the filterable field of the
                            // corresponding Texture entry above.
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                    label: Some("Renderer_texture_bind_group_layout"),
                });

        let render_pipeline_layout =
            gpu.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),

                    bind_group_layouts: &[&texture_bind_group_layout],
                    push_constant_ranges: &[],
                });

        let render_pipeline = gpu
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",     // 1.
                    buffers: &[Vertex::desc()], // 2.
                },
                fragment: Some(wgpu::FragmentState {
                    // 3.
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        // 4.
                        format: gpu.config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw, // 2.
                    cull_mode: Some(wgpu::Face::Back),
                    // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                    polygon_mode: wgpu::PolygonMode::Fill,
                    // Requires Features::DEPTH_CLIP_CONTROL
                    unclipped_depth: false,
                    // Requires Features::CONSERVATIVE_RASTERIZATION
                    conservative: false,
                },
                depth_stencil: None, // 1.
                multisample: wgpu::MultisampleState {
                    count: 1,                         // 2.
                    mask: !0,                         // 3.
                    alpha_to_coverage_enabled: false, // 4.
                },
                multiview: None, // 5.
            });

        let vertex_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let index_buffer: Buffer =
            gpu.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index buffer"),
                    contents: bytemuck::cast_slice(&INDICES),
                    usage: wgpu::BufferUsages::INDEX,
                });
        let num_indices = INDICES.len() as u32;

        Self {
            render_pipeline,
            texture_bind_group_layout,
            vertex_buffer,
            index_buffer,
            num_indices,
            sampler,
            size,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>, gpu: &mut GPUInterface) {
        if new_size.width > 0 && new_size.height > 0 {
            gpu.size = new_size;
            gpu.config.width = new_size.width;
            gpu.config.height = new_size.height;
            gpu.surface.configure(&gpu.device, &gpu.config);
            self.size = UVec2::new(new_size.width as u32, new_size.height as u32);
        }
    }

    pub fn handle_events(&mut self, event: &Event<()>) {
        //self.gui.handle_events(event);
    }

    pub fn render(
        &mut self,
        gpu: &GPUInterface,
        mandelbrot_texture: &wgpu::Texture,
    ) -> Result<(), wgpu::SurfaceError> {
        let output = gpu.surface.get_current_texture().unwrap();
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let texture_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &mandelbrot_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ), // CHANGED!
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler), // CHANGED!
                },
            ],
            label: Some("Texture bind group"),
        });

        {
            // 1.
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[
                    // This is what [[location(0)]] in the fragment shader targets
                    Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: true,
                        },
                    }),
                ],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &texture_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }
        gpu.queue.submit([encoder.finish()]);
        output.present();
        Ok(())
    }
}
