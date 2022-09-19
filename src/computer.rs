use bytemuck::{Pod, Zeroable};
use wgpu::{util::DeviceExt, BufferBindingType, Extent3d, TextureFormat, TextureSampleType};

use crate::{
    gpu_interface::GPUInterface,
    math::{FVec2, UVec2},
};

#[derive(Debug)]
pub struct SampleLocation {
    position: FVec2,
    zoom: f32,
    move_speed: f32,
}

impl Default for SampleLocation {
    fn default() -> Self {
        Self {
            position: FVec2 { x: 0.0, y: 0.0 },
            zoom: 1.0,
            move_speed: 0.05,
        }
    }
}

impl SampleLocation {
    pub fn to_mandlebrot_params(&self, max_iterations: i32) -> MandelbrotParams {
        let x_min = self.position.x - (self.zoom);
        let x_max = self.position.x + (self.zoom);
        let y_min = self.position.y - (self.zoom);
        let y_max = self.position.y + (self.zoom);
        MandelbrotParams {
            x_min,
            x_max,
            y_min,
            y_max,
            max_iterations,
        }
    }

    pub fn left(&mut self) {
        self.position.x -= self.zoom * self.move_speed;
    }

    pub fn right(&mut self) {
        self.position.x += self.zoom * self.move_speed;
    }

    pub fn up(&mut self) {
        self.position.y -= self.zoom * self.move_speed;
    }

    pub fn down(&mut self) {
        self.position.y += self.zoom * self.move_speed;
    }

    pub fn zoom_in(&mut self) {
        self.zoom *= 0.5;
    }

    pub fn zoom_out(&mut self) {
        self.zoom *= 2.0;
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct MandelbrotParams {
    pub x_min: f32,
    pub x_max: f32,
    pub y_min: f32,
    pub y_max: f32,
    pub max_iterations: i32,
}

pub struct Computer {
    pipeline: wgpu::ComputePipeline,
    output_texture: wgpu::Texture,
    texture_size: Extent3d,
}

impl Computer {
    pub fn new(size: UVec2, gpu: &GPUInterface) -> Computer {
        let texture_size = wgpu::Extent3d {
            width: size.x,
            height: size.y,
            depth_or_array_layers: 1,
        };
        let output_texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("output texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::TEXTURE_BINDING,
        });

        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Mandelbrot shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders/mandelbrot.wgsl").into()),
            });

        let pipeline = gpu
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Mandelbrot compute pipeline"),
                layout: None,
                module: &shader,
                entry_point: "main",
            });

        Computer {
            pipeline,
            output_texture,
            texture_size,
        }
    }

    pub fn run(&self, gpu: &GPUInterface, mandelbot_params: &MandelbrotParams) -> &wgpu::Texture {
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let m_params_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Params Buffer"),
                contents: bytemuck::bytes_of(mandelbot_params),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let compute_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Compute bind group"),
            layout: &self.pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &self
                            .output_texture
                            .create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: m_params_buffer.as_entire_binding(),
                },
            ],
        });

        {
            let (dispatch_with, dispatch_height) = compute_work_group_count(
                (self.texture_size.width, self.texture_size.height),
                (16, 16),
            );
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Grayscale pass"),
            });
            compute_pass.set_pipeline(&self.pipeline);
            compute_pass.set_bind_group(0, &compute_bind_group, &[]);
            compute_pass.dispatch_workgroups(dispatch_with, dispatch_height, 1);
        }

        // Get the result.
        /*
        println!("Finished computing. Saving file...");
        let padded_bytes_per_row = padded_bytes_per_row(self.texture_size.width);
        let unpadded_bytes_per_row = self.texture_size.width as usize * 4;

        let output_buffer_size = padded_bytes_per_row as u64
            * self.texture_size.height as u64
            * std::mem::size_of::<u8>() as u64;
        let output_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: output_buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &self.output_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            wgpu::ImageCopyBuffer {
                buffer: &output_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: std::num::NonZeroU32::new(padded_bytes_per_row as u32),
                    rows_per_image: std::num::NonZeroU32::new(self.texture_size.height),
                },
            },
            self.texture_size,
        );


        let buffer_slice = output_buffer.slice(..);
        let mapping = buffer_slice.map_async(wgpu::MapMode::Read, |a| {});

        gpu.device.poll(wgpu::Maintain::Wait);

        let padded_data = buffer_slice.get_mapped_range();

        let mut pixels: Vec<u8> =
            vec![0; unpadded_bytes_per_row * self.texture_size.height as usize];
        for (padded, pixels) in padded_data
            .chunks_exact(padded_bytes_per_row)
            .zip(pixels.chunks_exact_mut(unpadded_bytes_per_row))
        {
            pixels.copy_from_slice(&padded[..unpadded_bytes_per_row]);
        }

        if let Some(output_image) = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(
            self.texture_size.width,
            self.texture_size.height,
            &pixels[..],
        ) {
            output_image.save("output.png").unwrap();
        }*/
        gpu.queue.submit(Some(encoder.finish()));
        &self.output_texture
    }
}

fn compute_work_group_count(
    (width, height): (u32, u32),
    (workgroup_width, workgroup_height): (u32, u32),
) -> (u32, u32) {
    let x = (width + workgroup_width - 1) / workgroup_width;
    let y = (height + workgroup_height - 1) / workgroup_height;

    (x, y)
}

/// Compute the next multiple of 256 for texture retrieval padding.
fn padded_bytes_per_row(width: u32) -> usize {
    let bytes_per_row = width as usize * 4;
    let padding = (256 - bytes_per_row % 256) % 256;
    bytes_per_row + padding
}
