//! The path tracer on the GPU: `trace.wgsl` (the CPU's [`crate::Renderer::pass`] as a compute
//! shader) through wgpu on its own device (Vulkan, Metal or DX12), opened once, lazily.
//!
//! - [`available`] is false without a GPU (a software adapter doesn't count), with
//!   `CADRS_RENDER_GPU=0` in the environment, or when the shader fails to build; renders then
//!   stay on the CPU.
//! - A [`Scene`]'s buffers (the BVH, the triangles' shading data, the materials) are uploaded
//!   once and kept with it ([`Scene::gpu`]), so a new camera on the same scene uploads nothing.
//! - A [`Pass`] holds one render's sums on the GPU; each sample is dispatched in bands of rows
//!   (a long dispatch trips the drivers' watchdogs) and waited for. Reading the image back
//!   copies the sums to the CPU, which denoises and composes them as it does its own.
//! - Bigger than the device allows (a huge assembly, a huge image): no pass, so the CPU renders.

use std::sync::{Arc, OnceLock};

use wgpu::util::DeviceExt;

use crate::math::V3;
use crate::{Accum, Camera, Guide, Projection, Scene, Settings, env};

/// Most lights an environment has (`trace.wgsl`'s `Params.lights`).
const MAX_LIGHTS: usize = 8;
/// Pixels per dispatch.
const BAND_PIXELS: u32 = 1 << 18;
const ACCUM_FLOATS: usize = 10;
const GUIDE_FLOATS: usize = 12;

struct Context {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::ComputePipeline,
    limits: wgpu::Limits,
    name: String,
}

static CONTEXT: OnceLock<Option<Context>> = OnceLock::new();

fn context() -> Option<&'static Context> {
    CONTEXT.get_or_init(open).as_ref()
}

/// True when renders can run on the GPU.
pub fn available() -> bool {
    context().is_some()
}

/// The GPU renders run on, if any.
pub fn adapter_name() -> Option<&'static str> {
    context().map(|c| c.name.as_str())
}

fn open() -> Option<Context> {
    if std::env::var("CADRS_RENDER_GPU").is_ok_and(|v| v == "0" || v.eq_ignore_ascii_case("off")) {
        return None;
    }
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter =
        futures_lite::future::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            ..Default::default()
        }))
        .ok()?;
    let info = adapter.get_info();
    if info.device_type == wgpu::DeviceType::Cpu {
        return None;
    }
    let limits = adapter.limits();
    let (device, queue) =
        futures_lite::future::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("cadrs_render"),
            required_limits: limits.clone(),
            ..Default::default()
        }))
        .ok()?;
    // A device error is reported, not fatal (wgpu's default handler panics).
    device.on_uncaptured_error(Arc::new(|e: wgpu::Error| {
        eprintln!("cadrs_render: GPU error: {e}");
    }));
    let scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("cadrs_render trace"),
        source: wgpu::ShaderSource::Wgsl(include_str!("trace.wgsl").into()),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("cadrs_render trace"),
        layout: None,
        module: &module,
        entry_point: Some("main"),
        compilation_options: Default::default(),
        cache: None,
    });
    if let Some(e) = futures_lite::future::block_on(scope.pop()) {
        eprintln!("cadrs_render: the GPU path tracer didn't build: {e}");
        return None;
    }
    Some(Context {
        device,
        queue,
        pipeline,
        limits,
        name: info.name,
    })
}

/// A scene's buffers on the GPU.
pub(crate) struct GpuScene {
    nodes: wgpu::Buffer,
    tris: wgpu::Buffer,
    shade: wgpu::Buffer,
    materials: wgpu::Buffer,
    n_tris: u32,
}

fn storage(c: &Context, label: &str, data: &[f32]) -> Option<wgpu::Buffer> {
    // Bindings can't be empty.
    let data: &[f32] = if data.is_empty() { &[0.0; 4] } else { data };
    let bytes: &[u8] = bytemuck::cast_slice(data);
    if bytes.len() as u64 > c.limits.max_storage_buffer_binding_size
        || bytes.len() as u64 > c.limits.max_buffer_size
    {
        return None;
    }
    Some(
        c.device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(label),
                contents: bytes,
                usage: wgpu::BufferUsages::STORAGE,
            }),
    )
}

fn push3(out: &mut Vec<f32>, v: V3, w: f32) {
    out.extend_from_slice(&[v.x, v.y, v.z, w]);
}

impl GpuScene {
    fn upload(c: &Context, scene: &Scene) -> Option<GpuScene> {
        let bvh = &scene.bvh;
        let mut nodes = Vec::with_capacity(bvh.nodes.len() * 8);
        for n in &bvh.nodes {
            push3(&mut nodes, n.bounds.min, f32::from_bits(n.first));
            push3(&mut nodes, n.bounds.max, f32::from_bits(n.count));
        }
        let mut tris = Vec::with_capacity(bvh.tris.len() * 12);
        for t in &bvh.tris {
            push3(&mut tris, t.v0, 0.0);
            push3(&mut tris, t.e1, 0.0);
            push3(&mut tris, t.e2, 0.0);
        }
        let mut shade = Vec::with_capacity(bvh.order.len() * 16);
        for &i in &bvh.order {
            let i = i as usize;
            for v in scene.tris[i] {
                push3(&mut shade, scene.normals[v as usize], 0.0);
            }
            push3(
                &mut shade,
                scene.face_normals[i],
                f32::from_bits(scene.tri_material[i]),
            );
        }
        let mut materials = Vec::with_capacity(scene.materials.len() * 8);
        for m in &scene.materials {
            let [r, g, b] = m.base_color;
            materials.extend_from_slice(&[r, g, b, m.metallic, m.roughness, m.opacity, 0.0, 0.0]);
        }
        Some(GpuScene {
            nodes: storage(c, "render nodes", &nodes)?,
            tris: storage(c, "render triangles", &tris)?,
            shade: storage(c, "render shading", &shade)?,
            materials: storage(c, "render materials", &materials)?,
            n_tris: bvh.tris.len() as u32,
        })
    }
}

/// A [`GpuScene`] as a [`Scene`] keeps it: type-erased, so that the scene's `Send` and `Sync`
/// don't send the compiler through wgpu's types.
pub(crate) type Kept = Option<Box<dyn std::any::Any + Send + Sync>>;

impl Scene {
    /// The scene's GPU buffers, uploaded on first use; `None` without a GPU or when it doesn't
    /// fit.
    fn gpu(&self) -> Option<&GpuScene> {
        self.gpu
            .get_or_init(|| {
                GpuScene::upload(context()?, self)
                    .map(|g| Box::new(g) as Box<dyn std::any::Any + Send + Sync>)
            })
            .as_ref()?
            .downcast_ref()
    }
}

/// One render's sums on the GPU.
pub(crate) struct Pass {
    params: wgpu::Buffer,
    accum: wgpu::Buffer,
    guide: wgpu::Buffer,
    read: wgpu::Buffer,
    bind: wgpu::BindGroup,
    /// The uniform block, the per-dispatch words patched in.
    block: Vec<u32>,
    width: u32,
    height: u32,
}

/// Words of `trace.wgsl`'s `Params` before its lights.
const HEAD_WORDS: usize = 40;
const BLOCK_WORDS: usize = HEAD_WORDS + MAX_LIGHTS * 8;
// The words patched per dispatch: `dims.z` (the sample), `band.x` and `band.y`.
const SAMPLE_WORD: usize = 30;
const ROW0_WORD: usize = 36;
const ROWS_WORD: usize = 37;

impl Pass {
    pub(crate) fn new(
        scene: &Scene,
        camera: &Camera,
        settings: &Settings,
        env: &env::Environment,
        ground_z: f32,
        eps: f32,
    ) -> Option<Pass> {
        if env.lights.len() > MAX_LIGHTS {
            return None;
        }
        let c = context()?;
        let gs = scene.gpu()?;
        let (w, h) = (settings.width, settings.height);
        let n = w as u64 * h as u64;
        let accum_bytes = n * (ACCUM_FLOATS as u64) * 4;
        let guide_bytes = n * (GUIDE_FLOATS as u64) * 4;
        let max = c
            .limits
            .max_storage_buffer_binding_size
            .min(c.limits.max_buffer_size);
        if accum_bytes > max
            || guide_bytes > max
            || accum_bytes + guide_bytes > c.limits.max_buffer_size
        {
            return None;
        }

        let f = |x: f32| x.to_bits();
        let mut block = vec![0u32; BLOCK_WORDS];
        let mut put = |at: usize, v: [f32; 3], w: f32| {
            block[at] = f(v[0]);
            block[at + 1] = f(v[1]);
            block[at + 2] = f(v[2]);
            block[at + 3] = f(w);
        };
        let (kind, lens) = match camera.projection {
            Projection::Perspective { fov_y } => (0.0, (fov_y.to_radians() / 2.0).tan()),
            Projection::Orthographic { height } => (1.0, height),
        };
        put(0, camera.eye, kind);
        put(4, camera.forward, lens);
        put(8, camera.right, w as f32 / h as f32);
        put(12, camera.up, ground_z);
        put(16, env.zenith.to_array(), eps);
        put(20, env.horizon.to_array(), 0.0);
        put(24, env.ground.to_array(), 0.0);
        block[28] = w;
        block[29] = h;
        block[30] = 0;
        block[31] = settings.max_bounces;
        block[32] = settings.seed as u32;
        block[33] = (settings.seed >> 32) as u32;
        block[34] = env.lights.len() as u32;
        block[35] =
            u32::from(settings.ground_shadow) | (u32::from(env.backdrop(0.5, 0.5).is_none()) << 1);
        block[36] = 0;
        block[37] = 0;
        block[38] = gs.n_tris;
        for (i, l) in env.lights.iter().enumerate() {
            let at = HEAD_WORDS + i * 8;
            for (k, x) in [
                l.dir.x,
                l.dir.y,
                l.dir.z,
                l.cos_max,
                l.radiance.x,
                l.radiance.y,
                l.radiance.z,
                l.solid_angle,
            ]
            .into_iter()
            .enumerate()
            {
                block[at + k] = f(x);
            }
        }

        let d = &c.device;
        let params = d.create_buffer(&wgpu::BufferDescriptor {
            label: Some("render params"),
            size: (BLOCK_WORDS * 4) as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        // Zeroed on creation.
        let sums = |label, size| {
            d.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            })
        };
        let accum = sums("render accum", accum_bytes);
        let guide = sums("render guide", guide_bytes);
        let read = d.create_buffer(&wgpu::BufferDescriptor {
            label: Some("render readback"),
            size: accum_bytes + guide_bytes,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let entries: Vec<wgpu::BindGroupEntry> = [
            &params,
            &gs.nodes,
            &gs.tris,
            &gs.shade,
            &gs.materials,
            &accum,
            &guide,
        ]
        .into_iter()
        .enumerate()
        .map(|(i, b)| wgpu::BindGroupEntry {
            binding: i as u32,
            resource: b.as_entire_binding(),
        })
        .collect();
        let bind = d.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("render"),
            layout: &c.pipeline.get_bind_group_layout(0),
            entries: &entries,
        });
        Some(Pass {
            params,
            accum,
            guide,
            read,
            bind,
            block,
            width: w,
            height: h,
        })
    }

    /// Adds sample `s` to every pixel, and waits for it.
    pub(crate) fn pass(&mut self, s: u32) {
        let Some(c) = context() else { return };
        let rows = (BAND_PIXELS / self.width.max(1)).max(1);
        let mut row0 = 0;
        while row0 < self.height {
            let n = rows.min(self.height - row0);
            self.block[SAMPLE_WORD] = s;
            self.block[ROW0_WORD] = row0;
            self.block[ROWS_WORD] = n;
            c.queue
                .write_buffer(&self.params, 0, bytemuck::cast_slice(&self.block));
            let mut enc = c.device.create_command_encoder(&Default::default());
            {
                let mut pass = enc.begin_compute_pass(&Default::default());
                pass.set_pipeline(&c.pipeline);
                pass.set_bind_group(0, &self.bind, &[]);
                pass.dispatch_workgroups(self.width.div_ceil(8), n.div_ceil(8), 1);
            }
            c.queue.submit([enc.finish()]);
            row0 += n;
        }
        let _ = c.device.poll(wgpu::PollType::wait_indefinitely());
    }

    /// The sums so far, and the guides if `guides` (else none).
    pub(crate) fn read(&self, guides: bool) -> (Vec<Accum>, Vec<Guide>) {
        let n = self.width as usize * self.height as usize;
        let Some(c) = context() else {
            return (vec![Accum::default(); n], vec![Guide::default(); n]);
        };
        let (ab, gb) = (self.accum.size(), self.guide.size());
        let mut enc = c.device.create_command_encoder(&Default::default());
        enc.copy_buffer_to_buffer(&self.accum, 0, &self.read, 0, ab);
        if guides {
            enc.copy_buffer_to_buffer(&self.guide, 0, &self.read, ab, gb);
        }
        c.queue.submit([enc.finish()]);
        let slice = self.read.slice(..if guides { ab + gb } else { ab });
        slice.map_async(wgpu::MapMode::Read, |_| {});
        let _ = c.device.poll(wgpu::PollType::wait_indefinitely());
        let (accum, guide) = {
            let bytes = slice.get_mapped_range().expect("the readback is mapped");
            let floats: &[f32] = bytemuck::cast_slice(&bytes[..]);
            let (a, g) = floats.split_at(n * ACCUM_FLOATS);
            let g = if guides { g } else { &[] };
            let accum = a
                .as_chunks::<ACCUM_FLOATS>()
                .0
                .iter()
                .map(|x| Accum {
                    model: [x[0], x[1], x[2]],
                    n_model: x[3],
                    sky: [x[4], x[5], x[6]],
                    n_ground: x[7],
                    occluded: x[8],
                    open: x[9],
                })
                .collect();
            let guide = g
                .as_chunks::<GUIDE_FLOATS>()
                .0
                .iter()
                .map(|x| Guide {
                    normal: V3::from_array([x[0], x[1], x[2]]),
                    albedo: V3::from_array([x[3], x[4], x[5]]),
                    position: V3::from_array([x[6], x[7], x[8]]),
                    footprint: x[9],
                    hit: x[10] > 0.5,
                })
                .collect();
            (accum, guide)
        };
        self.read.unmap();
        (accum, guide)
    }
}
