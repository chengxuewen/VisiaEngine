//! The mesh pipeline core: device ownership + mesh/material resource tables + `frame -> any view` drawing.
//! Double consumers (DRY): `HeadlessBackend` (L1 offscreen) and `examples/` (L2 window).

use std::collections::HashMap;

use visiaengine_core::srgb_to_linear;
use wgpu::util::DeviceExt as _;

use visiaengine_render::{
    BackendError, Capability, DrawCommand, Frame, InstanceDesc, LabelTableDesc, MaterialDesc,
    MaterialId, MeshDesc, MeshId, PointTableDesc, StrokeTableDesc, TableId, TextureDesc, TextureId,
    Viewport,
};

struct GpuMesh {
    vbuf: wgpu::Buffer,
    ibuf: wgpu::Buffer,
    index_count: u32,
}

/// The render core without surface dependency; the target view is supplied per frame by the caller.
pub struct MeshCore {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    /// attach/surface reuse the same instance+adapter (the device-pairing constraint, I3)
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    meshes: HashMap<MeshId, GpuMesh>,
    materials: HashMap<MaterialId, MatGpu>,
    next_id: u64,
    /// Texture slots (WGPU-14): tex id -> (texture, view); the sampler is a global singleton (REPEAT/linear)
    textures: HashMap<u64, (wgpu::Texture, wgpu::TextureView)>,
    next_tex: u64,
    sampler: Option<wgpu::Sampler>,
    /// Instance table (WGPU-16): id -> (storage buffer, instance count)
    instances: HashMap<u64, GpuInstances>,
    /// Line-segment table (WGPU-17): id -> (storage buffer, segment count)
    strokes_t: HashMap<TableId, GpuInstances>,
    /// Point table (WGPU-18): id -> (storage buffer, point count)
    points_t: HashMap<TableId, GpuInstances>,
    bgl_label: wgpu::BindGroupLayout,
    layout_label: wgpu::PipelineLayout,
    /// Label table (WGPU-24): id -> (storage buffer, quad count)
    labels_t: HashMap<TableId, GpuInstances>,
    /// Glyph atlas single slot (WGPU-24): R8 texture + view (io-text full upload, None = skip even when the table is present)
    glyph_atlas: Option<(wgpu::Texture, wgpu::TextureView)>,
    /// Atlas sampler (ClampToEdge + Linear, resident)
    label_smp: wgpu::Sampler,
    /// The shared tile-expansion quad (side in ±1, 2 triangles) -- the standing vertex source for strokes/points
    quad_vb: wgpu::Buffer,
    quad_ib: wgpu::Buffer,
    /// (color format, variant) -> pipeline (three layouts, three pipeline families: the mega-bool veto [E3D:B2];
    /// the format dimension = the I3 surface legacy [Momus-R3]; Instanced = 4c/WGPU-16)
    pipelines: HashMap<(wgpu::TextureFormat, Variant), wgpu::RenderPipeline>,
    shader: wgpu::ShaderModule,
    layout: wgpu::PipelineLayout,
    layout_tex: wgpu::PipelineLayout,
    layout_inst: wgpu::PipelineLayout,
    layout_stroke: wgpu::PipelineLayout,
    layout_point: wgpu::PipelineLayout,
    bgl: wgpu::BindGroupLayout,
    bgl_tex: wgpu::BindGroupLayout,
    bgl_inst: wgpu::BindGroupLayout,
    bgl_stroke: wgpu::BindGroupLayout,
    bgl_point: wgpu::BindGroupLayout,
    /// Depth-face cache (WGPU-13: rebuild on size change; the pipeline and the attachment format share one source)
    depth_cache: Option<(u32, u32, wgpu::Texture, wgpu::TextureView)>,
    /// WGPU-19: shadow map (1024^2, lazily built and resident) + dummy 1x1 (the None position always binds) + comparison sampler
    /// + params-off UBO (enabled = 0, light_dir carries the default position = the old LIGHT constants bitwise)
    shadow_map: Option<(wgpu::Texture, wgpu::TextureView)>,
    sh_dummy: wgpu::TextureView,
    sh_sampler: wgpu::Sampler,
    sh_off: wgpu::Buffer,
    layout_shadow: wgpu::PipelineLayout,
    layout_shadow_inst: wgpu::PipelineLayout,
    bgl_shadow: wgpu::BindGroupLayout,
    bgl_shadow_inst: wgpu::BindGroupLayout,
    /// WGPU-33: EDL post-pass state (built always, cheap; color intermediate is
    /// size+format cached). edl_color = intermediate color view (main pass
    /// writes it when EDL is on, EDL pass samples it); edl_bgl = 4 bindings
    /// (color tex / depth tex / sampler / params UBO texel_w,texel_h,strength,0).
    edl_color: Option<(u32, u32, wgpu::TextureView, wgpu::TextureFormat)>,
    edl_bgl: wgpu::BindGroupLayout,
    edl_pipeline: HashMap<wgpu::TextureFormat, wgpu::RenderPipeline>,
    edl_smp: wgpu::Sampler,
    edl_shader: wgpu::ShaderModule,
    /// WGPU-34: post chain state. Same 4-binding bgl shape as edl (bloom
    /// ignores depth tex), shared edl_smp sampler, per-effect shaders and
    /// per-(effect,format) pipelines. Stage color intermediates = fixed 2-slot
    /// array keyed (effect index in chain) — chains longer than 2 are a plan
    /// non-goal this band.
    post_bgl: wgpu::BindGroupLayout,
    post_shaders: [wgpu::ShaderModule; 5], // [bloom, outline, tonemap, ssao, haze]
    /// I band (WGPU-39): SH-9 environment irradiance (engine-level state,
    /// survives across frames; zero vector = legacy constant ambient).
    env_sh: [[f32; 3]; 9],
    post_pipelines: [HashMap<wgpu::TextureFormat, wgpu::RenderPipeline>; 5],
    post_stages: [Option<(u32, u32, wgpu::TextureView, wgpu::TextureFormat)>; 2],
}

/// The material GPU state: a 32B uniform block + an optional texture view (the textured-variant key source).
struct MatGpu {
    buf: wgpu::Buffer,
    textured: bool,
    view: Option<wgpu::TextureView>,
    /// Source of the transparency predicate (4): material alpha (host face = the raw a channel, passed through by srgb_to_linear).
    /// alpha < 1 => the piece rides the transparent pipeline (REND-36 single decision; material a == 1 with texel a < 1 rides the old path = declared).
    alpha: f32,
}

/// Pipeline variant (the REND-27 key source; the Textured x Instanced combination is not installed [four-not-installed 2]).
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Variant {
    Flat,
    Textured,
    Instanced,
    Strokes,
    Points,
    /// Text-label patch (WGPU-24/S2): atlas coverage x table color, screen-constant size (bgl = 0+11+12+13).
    Labels,
    /// The WGPU-19 caster pass (no color target; Mesh = DrawMesh source, Inst = DrawInstances source)
    ShadowMesh,
    ShadowInst,
    /// The transparent trio (4) (REND-36/WGPU-27): blend = the SRC_ALPHA standard form (Labels same equation) +
    /// depthWrite = false / compare stays Less; the layout shares its source with the opaque, zero extension (the WGPU-14 zero-number-breakage precedent, same system).
    FlatT,
    TexturedT,
    InstancedT,
}

/// The 5b multi-viewport clear policy (default = the safe form; AllClear only as a ruling probe).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MultiClearPolicy {
    /// First pass Clear (full-frame background), subsequent Load -- the wgpu30 safe form (WGPU-26 measured lock).
    FirstClearRestLoad,
    /// Every pass Clears (Clear respects area means mutual storage; full-region means the latter swallows the former -- the divergence probe).
    AllClear,
}

/// The GPU instance table (WGPU-16): a 32B-per-entry storage buffer + entry count.
struct GpuInstances {
    buf: wgpu::Buffer,
    count: u32,
}

/// The depth pipeline format (wgpu native [0,1], the rightful side of the PIT-5 discipline)
pub(crate) const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

/// Request the PRIMARY adapter without a surface and build the device; failure = None (reportable semantics).
#[must_use]
pub struct Shared {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
}

/// Request the PRIMARY adapter without a surface and build the device (instance/adapter returned -- the surface pairing constraint);
/// failure = None (reportable semantics).
#[must_use]
pub fn create_shared_device() -> Option<Shared> {
    pollster::block_on(create_shared_device_async())
}

/// The async body (the web main thread forbids blocking, batch 7 J1; native runs the pollster equivalent shell).
pub async fn create_shared_device_async() -> Option<Shared> {
    let instance = crate::create_instance();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            ..Default::default()
        })
        .await
        .ok()?;
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("visiaengine-core-dev"),
            required_features: wgpu::Features::empty(),
            required_limits: adapter.limits(),
            ..Default::default()
        })
        .await
        .ok()?;
    Some(Shared {
        device,
        queue,
        instance,
        adapter,
    })
}

impl MeshCore {
    pub fn new(
        device: wgpu::Device,
        queue: wgpu::Queue,
        instance: wgpu::Instance,
        adapter: wgpu::Adapter,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("mesh-shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                "../shaders/mesh.wgsl"
            ))),
        });
        // material block 48B v2 (WGPU-35/REND-45: base_color+repeat+roughness+
        // metallic+pad; binding2 both layouts same size — all variants read the
        // same block = single-truth construction guarantee [Momus-B1 shape])
        let mk_bgl = |label: &'static str, variant: Variant| {
            let mut entries = vec![
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        // the View block is 256B (WGPU-21 including the clip-plane segment; the first 176B keep the position order)
                        min_binding_size: wgpu::BufferSize::new(416),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(48),
                    },
                    count: None,
                },
            ];
            match variant {
                // The T-family bgl is identical across the family (layout zero extension [WGPU-27]; the exhaustive synchronizer names this arm live)
                Variant::Flat | Variant::FlatT => {}
                Variant::Textured | Variant::TexturedT => {
                    entries.push(wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    });
                    entries.push(wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    });
                }
                Variant::Instanced | Variant::InstancedT => {
                    entries.push(wgpu::BindGroupLayoutEntry {
                        binding: 5,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(32),
                        },
                        count: None,
                    })
                }
                // WGPU-18 point-table storage (32B per entry minimum, binding6; materials do not participate, same as ruling point a)
                Variant::Points => {
                    entries.remove(1);
                    entries.push(wgpu::BindGroupLayoutEntry {
                        binding: 6,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(32),
                        },
                        count: None,
                    });
                }
                // WGPU-19 receiver trio appended after the match uniformly (dummy residency = a dynamic branch, not a layout branch)
                Variant::ShadowMesh | Variant::ShadowInst => {
                    unreachable!("the caster bgl is built separately")
                }
                // WGPU-24 labels: table (11 storage 64B) + atlas texture (12) + sampler (13); materials not attached [ruling a]
                Variant::Labels => {
                    entries.remove(1);
                    entries.extend_from_slice(&[
                        wgpu::BindGroupLayoutEntry {
                            binding: 11,
                            visibility: wgpu::ShaderStages::VERTEX,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: wgpu::BufferSize::new(64),
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 12,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 13,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ]);
                }
                // WGPU-17 line-table storage (48B per entry minimum); material binding2 does not participate [ruling point a]
                Variant::Strokes => {
                    entries.remove(1);
                    entries.push(wgpu::BindGroupLayoutEntry {
                        binding: 5,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(48),
                        },
                        count: None,
                    });
                }
            }
            if matches!(
                variant,
                Variant::Flat
                    | Variant::Textured
                    | Variant::Instanced
                    | Variant::FlatT
                    | Variant::TexturedT
                    | Variant::InstancedT
            ) {
                entries.extend_from_slice(&[
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(32),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 7,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Depth,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 8,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                        count: None,
                    },
                ]);
            }
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some(label),
                entries: &entries,
            })
        };
        // The WGPU-19 caster bgl pair (binding0 [+5]; for the pipeline with no color target)
        let mk_shadow_entries = |inst: bool| {
            let mut e = vec![wgpu::BindGroupLayoutEntry {
                binding: 0,
                // WGPU-22: fs_shadow consumes view.planes (the caster clipped the same) -> +FRAGMENT visible
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(416),
                },
                count: None,
            }];
            if inst {
                e.push(wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(32),
                    },
                    count: None,
                });
            }
            e
        };
        let bgl_shadow = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("mesh-bgl-shadow"),
            entries: &mk_shadow_entries(false),
        });
        let bgl_shadow_inst = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("mesh-bgl-shadow-inst"),
            entries: &mk_shadow_entries(true),
        });
        let bgl_flat = mk_bgl("mesh-bgl", Variant::Flat);
        let bgl_tex = mk_bgl("mesh-bgl-tex", Variant::Textured);
        let bgl_inst = mk_bgl("mesh-bgl-inst", Variant::Instanced);
        let bgl_stroke = mk_bgl("mesh-bgl-stroke", Variant::Strokes);
        let bgl_point = mk_bgl("mesh-bgl-point", Variant::Points);
        let bgl_label = mk_bgl("mesh-bgl-label", Variant::Labels);
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("mesh-pl"),
            bind_group_layouts: &[Some(&bgl_flat)],
            immediate_size: 0,
        });
        let layout_tex = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("mesh-pl-tex"),
            bind_group_layouts: &[Some(&bgl_tex)],
            immediate_size: 0,
        });
        let layout_inst = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("mesh-pl-inst"),
            bind_group_layouts: &[Some(&bgl_inst)],
            immediate_size: 0,
        });
        let layout_stroke = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("mesh-pl-stroke"),
            bind_group_layouts: &[Some(&bgl_stroke)],
            immediate_size: 0,
        });
        // The shared tile-expansion quad (the standing vertex source of the [E3D:B4] GS->VS form)
        let quad_vb = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("quad-vb"),
            contents: bytemuck::cast_slice(&[-1.0f32, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0]),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let quad_ib = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("quad-ib"),
            contents: bytemuck::cast_slice(&[0u32, 1, 2, 0, 2, 3]),
            usage: wgpu::BufferUsages::INDEX,
        });
        let layout_point = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("mesh-pl-point"),
            bind_group_layouts: &[Some(&bgl_point)],
            immediate_size: 0,
        });
        let layout_label = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("mesh-pl-label"),
            bind_group_layouts: &[Some(&bgl_label)],
            immediate_size: 0,
        });
        let layout_shadow = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("mesh-pl-shadow"),
            bind_group_layouts: &[Some(&bgl_shadow)],
            immediate_size: 0,
        });
        let layout_shadow_inst = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("mesh-pl-shadow-inst"),
            bind_group_layouts: &[Some(&bgl_shadow_inst)],
            immediate_size: 0,
        });
        let sh_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("shadow-cmp"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            compare: Some(wgpu::CompareFunction::Less),
            ..Default::default()
        });
        let dummy = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("shadow-dummy"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let sh_dummy = dummy.create_view(&wgpu::TextureViewDescriptor::default());
        let sh_off = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("shadow-off"),
            contents: bytemuck::cast_slice(&[0.5f32, 0.7, 0.4, 0.0, 0.0, 1.0, 0.0, 0.0]),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let mut pipelines = HashMap::new();
        for (variant, pl) in [
            (Variant::Flat, &layout),
            (Variant::Textured, &layout_tex),
            (Variant::Instanced, &layout_inst),
            (Variant::Strokes, &layout_stroke),
            (Variant::Points, &layout_point),
        ] {
            pipelines.insert(
                (wgpu::TextureFormat::Rgba8Unorm, variant),
                build_pipeline(
                    &device,
                    pl,
                    &shader,
                    wgpu::TextureFormat::Rgba8Unorm,
                    variant,
                ),
            );
        }
        for (variant, pl) in [
            (Variant::ShadowMesh, &layout_shadow),
            (Variant::ShadowInst, &layout_shadow_inst),
        ] {
            pipelines.insert(
                (wgpu::TextureFormat::Rgba8Unorm, variant),
                build_pipeline(
                    &device,
                    pl,
                    &shader,
                    wgpu::TextureFormat::Rgba8Unorm,
                    variant,
                ),
            );
        }
        let _ = dummy;
        // WGPU-33: EDL post-pass statics — bgl (4 bindings), nearest clamp sampler,
        // shader module. Pipeline is per-format lazy (edl_pipeline map); params UBO
        // and intermediate color target are built on first EDL-on frame (cache fields).
        let edl_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("edl-shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                "../shaders/edl.wgsl"
            ))),
        });
        // Shared 4-binding entry shape: EDL (WGPU-33) and post chain (WGPU-34)
        // use the same layout (color tex / depth tex / sampler / params UBO).
        let fullscreen_entries = &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Depth,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ];
        let edl_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("edl-bgl"),
            entries: fullscreen_entries,
        });
        let edl_smp = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("edl-smp"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        // WGPU-34: post chain statics — shared 4-binding bgl (same shape as
        // edl_bgl; bloom samples only color but binds depth for shape sharing),
        // edl_smp reused (same nearest-clamp shape), one shader module per effect.
        let post_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("post-bgl"),
            entries: fullscreen_entries,
        });
        let post_shaders = [
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("bloom-shader"),
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                    "../shaders/bloom.wgsl"
                ))),
            }),
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("outline-shader"),
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                    "../shaders/outline.wgsl"
                ))),
            }),
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("tonemap-shader"),
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                    "../shaders/tonemap.wgsl"
                ))),
            }),
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("ssao-shader"),
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                    "../shaders/ssao.wgsl"
                ))),
            }),
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("haze-shader"),
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                    "../shaders/haze.wgsl"
                ))),
            }),
        ];
        let label_smp = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("atlas-smp"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        Self {
            env_sh: [[0.0; 3]; 9],
            device,
            queue,
            instance,
            adapter,
            meshes: HashMap::new(),
            materials: HashMap::new(),
            instances: HashMap::new(),
            strokes_t: HashMap::new(),
            points_t: HashMap::new(),
            quad_vb,
            quad_ib,
            next_id: 0,
            textures: HashMap::new(),
            next_tex: 0,
            sampler: None,
            pipelines,
            shader,
            layout,
            layout_tex,
            layout_inst,
            layout_stroke,
            layout_point,
            bgl_label,
            layout_label,
            bgl: bgl_flat,
            bgl_tex,
            bgl_inst,
            bgl_stroke,
            bgl_point,
            labels_t: HashMap::new(),
            glyph_atlas: None,
            label_smp,
            depth_cache: None,
            shadow_map: None,
            sh_dummy,
            sh_sampler,
            sh_off,
            layout_shadow,
            layout_shadow_inst,
            bgl_shadow,
            bgl_shadow_inst,
            edl_color: None,
            edl_bgl,
            edl_pipeline: HashMap::new(),
            edl_smp,
            edl_shader,
            post_bgl,
            post_shaders,
            post_pipelines: [
                HashMap::new(),
                HashMap::new(),
                HashMap::new(),
                HashMap::new(),
                HashMap::new(),
            ],
            post_stages: [None, None],
        }
    }

    /// I band (WGPU-39): install SH-9 environment irradiance coefficients
    /// (linear RGB). Zero vector restores the legacy constant-ambient path.
    pub fn set_env_sh(&mut self, sh: [[f32; 3]; 9]) {
        self.env_sh = sh;
    }

    pub fn uniform(&self, bytes: &[u8], label: &'static str) -> wgpu::Buffer {
        let mut padded = bytes.to_vec();
        padded.resize(padded.len().div_ceil(256) * 256, 0);
        let buf = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: padded.len() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.queue.write_buffer(&buf, 0, &padded);
        buf
    }

    pub fn upload_mesh(&mut self, desc: &MeshDesc<'_>) -> Result<MeshId, BackendError> {
        if desc.positions.len() != desc.normals.len()
            || desc.indices.is_empty()
            || (!desc.uv.is_empty() && desc.uv.len() != desc.positions.len())
        {
            return Err(BackendError {
                reason: "MeshDesc length mismatch or empty indices".into(),
            });
        }
        let mut interleaved: Vec<f32> = Vec::with_capacity(desc.positions.len() * 8);
        for (idx, (p, n)) in desc.positions.iter().zip(desc.normals.iter()).enumerate() {
            interleaved.extend_from_slice(p);
            interleaved.extend_from_slice(n);
            let uv = desc.uv.get(idx).copied().unwrap_or([0.0, 0.0]);
            interleaved.extend_from_slice(&uv);
        }
        let vbuf = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("mesh-vb"),
            size: (interleaved.len() * 4) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.queue
            .write_buffer(&vbuf, 0, bytemuck::cast_slice(&interleaved));
        let ibuf = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("mesh-ib"),
            size: (desc.indices.len() * 4) as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.queue
            .write_buffer(&ibuf, 0, bytemuck::cast_slice(desc.indices));
        self.next_id += 1;
        let id = self.next_id;
        self.meshes.insert(
            id,
            GpuMesh {
                vbuf,
                ibuf,
                index_count: desc.indices.len() as u32,
            },
        );
        Ok(id)
    }

    pub fn upload_material(&mut self, base_color: [f32; 4]) -> Result<MaterialId, BackendError> {
        self.upload_material_desc(&MaterialDesc {
            base_color,
            texture: None,
            repeat: [1.0, 1.0],
            specular: 0.0,
            roughness: 1.0, // N6: dielectric legacy band (WGPU-35 probe ledger)
            metallic: 0.0,
        })
    }

    /// Material description version (WGPU-14 v2 / WGPU-35): 48B uniform block
    /// [base_color(16), repeat(8), roughness(4), metallic(4), pad(16)];
    /// texture=Some → textured variant (unknown texture id = Err, no silent
    /// Flat fallback).
    pub fn upload_material_desc(
        &mut self,
        desc: &MaterialDesc,
    ) -> Result<MaterialId, BackendError> {
        let (textured, view) = match desc.texture {
            Some(tid) => match self.textures.get(&tid) {
                Some((_, view)) => (true, Some(view.clone())),
                None => {
                    return Err(BackendError {
                        reason: "material references unknown texture".into(),
                    });
                }
            },
            None => (false, None),
        };
        // CORE-16 throat one: base_color host face = sRGB (CSS convention),
        // written = linear (CORE-16); GGX params pass through raw (WGPU-35).
        let lin = srgb_to_linear(desc.base_color);
        // 48B block v2 (REND-45). WGSL Mat alignment: base_color@0 (vec4,
        // align 16), repeat@16 (vec2), roughness@24, metallic@28, pad@32..48
        // — retired mock specular float slot is absorbed into the tail pad.
        let block = [
            lin[0],
            lin[1],
            lin[2],
            lin[3],
            desc.repeat[0],
            desc.repeat[1],
            desc.roughness,
            desc.metallic,
            0.0,
            0.0,
            0.0,
            0.0,
        ];
        let buf = self.uniform(bytemuck::cast_slice(&block), "material");
        self.next_id += 1;
        let id = self.next_id;
        self.materials.insert(
            id,
            MatGpu {
                buf,
                textured,
                view,
                alpha: lin[3], // the a channel passed through = the srgb_to_linear raw value [REND-36 single decision source]
            },
        );
        Ok(id)
    }

    /// RGBA8 -> GPU texture (WGPU-14): the row pitch zero-padded to 256 alignment (the write_texture constraint).
    pub fn upload_texture(&mut self, desc: &TextureDesc<'_>) -> Result<TextureId, BackendError> {
        if desc.width == 0 || desc.height == 0 {
            return Err(BackendError {
                reason: "zero-size texture".into(),
            });
        }
        let need = desc.width as usize * desc.height as usize * 4;
        if desc.rgba.len() < need {
            return Err(BackendError {
                reason: "texture data shorter than width*height*4".into(),
            });
        }
        let tex = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("diffuse"),
            size: wgpu::Extent3d {
                width: desc.width,
                height: desc.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            // CORE-16: the host texture bytes = sRGB encoded (the PNG/JPEG/bitmap convention), the Srgb format
            // = hardware-decodes into the linear lighting domain at sampling (same domain as the material block/clear, the K3 slice).
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let row = desc.width as usize * 4;
        let stride = row.next_multiple_of(256);
        let mut data = Vec::with_capacity(stride * desc.height as usize);
        if stride == row {
            data.extend_from_slice(&desc.rgba[..need]);
        } else {
            for y in 0..desc.height as usize {
                data.extend_from_slice(&desc.rgba[y * row..(y + 1) * row]);
                data.resize(data.len() + (stride - row), 0);
            }
        }
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(stride as u32),
                rows_per_image: Some(desc.height),
            },
            wgpu::Extent3d {
                width: desc.width,
                height: desc.height,
                depth_or_array_layers: 1,
            },
        );
        let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
        self.next_tex += 1;
        let id = self.next_tex;
        self.textures.insert(id, (tex, view));
        Ok(id)
    }

    /// Instance-table upload (WGPU-16): a 32B-per-entry storage table; empty tables rejected at build (the REND-27 closed ruling).
    pub fn create_instances(&mut self, desc: &InstanceDesc<'_>) -> Result<u64, BackendError> {
        if desc.data.is_empty() {
            return Err(BackendError {
                reason: "empty instance table".into(),
            });
        }
        let mut data = desc.data.to_vec(); // chokepoint five of CORE-16: the linearized copy of instance colors (the multiply chain stays in-domain)
        for inst in &mut data {
            inst.color = srgb_to_linear([inst.color[0], inst.color[1], inst.color[2], 1.0])[..3]
                .try_into()
                .expect("3ch");
        }
        let bytes = bytemuck::cast_slice(&data);
        let buf = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("instances"),
            size: bytes.len() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.queue.write_buffer(&buf, 0, bytes);
        self.next_id += 1;
        let id = self.next_id;
        self.instances.insert(
            id,
            GpuInstances {
                buf,
                count: desc.data.len() as u32,
            },
        );
        Ok(id)
    }

    /// The View uniform block is 128B (the REND-29 delivery; [f32;32] matches the WGSL struct layout):
    /// mat@0..16 / right@16 / up@20 / px_scale@23 / eye_local@24..27 (the world eye-origin in f32).
    /// right/up = view_rot row extraction -- the single source of truth sharing view_rot with screen_to_ray_*.
    fn view_block(
        &self,
        frame: &Frame,
        origin: &[f64; 3],
        transform: &[[f64; 4]; 4],
    ) -> [f32; 104] {
        let mvp = visiaengine_render::rebase::compose_mvp(
            &frame.proj,
            &frame.view_rot,
            &frame.eye,
            origin,
            transform,
        );
        let mut block = [0.0f32; 104];
        block[..16].copy_from_slice(bytemuck::cast_slice(&mvp));
        let r = &frame.view_rot;
        // column-major m[col][row]: world right = row 0 = (m00,m10,m20); up = row 1
        block[16] = r[0][0];
        block[17] = r[1][0];
        block[18] = r[2][0];
        block[20] = r[0][1];
        block[21] = r[1][1];
        block[22] = r[2][1];
        block[23] = frame.px_world_scale;
        block[24] = (frame.eye[0] - origin[0]) as f32;
        block[25] = (frame.eye[1] - origin[1]) as f32;
        block[26] = (frame.eye[2] - origin[2]) as f32;
        if let Some(s) = &frame.shadow {
            let lvp = visiaengine_render::rebase::compose_mvp(
                &s.proj,
                &s.view_rot,
                &s.eye,
                origin,
                transform,
            );
            block[28..44].copy_from_slice(bytemuck::cast_slice(&lvp));
        }
        // WGPU-21: per-draw plane-coefficient conversion (origin + transform are right in this function's signature --
        // the design dividend of no new binding/bind-group construction); None/EMPTY = all zeros = the shader early-out always passes.
        if let Some(cs) = &frame.clip {
            for (i, pl) in cs.planes[..cs.count].iter().enumerate() {
                let (nm, dl) = visiaengine_render::clip_to_local(*pl, *origin, transform);
                let o = 44 + i * 4;
                block[o] = nm[0];
                block[o + 1] = nm[1];
                block[o + 2] = nm[2];
                block[o + 3] = dl;
            }
            block[60] = cs.count as f32;
        }
        // I band (WGPU-39): SH-9 environment irradiance coefficients
        // (27 floats at 64..91; 92..96 pad). All-zero = engine default =
        // legacy constant-ambient path in the shader (bit-identical: the
        // shader branches on a baked flag in slot 64... kept simpler: the
        // zero SH vector evaluates to ambient 0 — the SH path REPLACES the
        // legacy ambient when env_sh_flag(91) is set, else legacy constant).
        for (i, c) in self.env_sh.iter().enumerate() {
            let o = 64 + i * 4;
            block[o] = c[0];
            block[o + 1] = c[1];
            block[o + 2] = c[2];
            // block[o + 3] stays 0 (vec4 pad component)
        }
        // env_on flag lives AFTER the 9xvec4 block: float 100 (byte 400).
        block[100] = if self.env_sh.iter().any(|c| *c != [0.0; 3]) {
            1.0
        } else {
            0.0
        };
        block
    }

    /// [4f section 2.4] The GL-backend downgrade gate (in wgpu-30 GLES = the Gl sub-version, one variant covers it):
    /// arriving means warn-once + the whole chain treated as None (the dummy/off resident positions, no panic no black screen).
    fn shadow_active(&self, frame: &Frame) -> bool {
        if frame.shadow.is_none() {
            return false;
        }
        if matches!(self.adapter.get_info().backend, wgpu::Backend::Gl) {
            static ONCE: std::sync::Once = std::sync::Once::new();
            ONCE.call_once(|| {
                eprintln!("warn: shadow unsupported on GL backend, disabled [4f downgrade]");
            });
            return false;
        }
        true
    }

    /// The frame-shared shadow resources (inactive = the dummy/off resident positions): returns (params buf, map view).
    fn shadow_frame_res(&mut self, frame: &Frame) -> (wgpu::Buffer, wgpu::TextureView) {
        let Some(s) = frame.shadow.filter(|_| self.shadow_active(frame)) else {
            return (self.sh_off.clone(), self.sh_dummy.clone());
        };
        const MAP: u32 = 1024;
        if self.shadow_map.is_none() {
            let tex = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("shadow-map"),
                size: wgpu::Extent3d {
                    width: MAP,
                    height: MAP,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: DEPTH_FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
            self.shadow_map = Some((tex, view));
        }
        let mview = self.shadow_map.as_ref().expect("just built").1.clone();
        let params = [
            s.light_dir[0],
            s.light_dir[1],
            s.light_dir[2],
            1.0, // enabled
            s.size,
            1.0 / MAP as f32,
            s.bias.constant,
            s.bias.slope,
        ];
        let buf = self.uniform(bytemuck::cast_slice(&params), "shadow-params");
        (buf, mview)
    }

    /// Line-table upload (WGPU-17): 48B per entry; empty tables rejected at build (the same discipline as 4c).
    pub fn create_strokes(&mut self, desc: &StrokeTableDesc<'_>) -> Result<TableId, BackendError> {
        if desc.data.is_empty() {
            return Err(BackendError {
                reason: "empty stroke table".into(),
            });
        }
        // chokepoint three of CORE-16: tile-expansion family colors = linearized copies into the table (the Pod contract unchanged, the consumption face = color-domain note)
        let mut data = desc.data.to_vec();
        for seg in &mut data {
            seg.color = srgb_to_linear([seg.color[0], seg.color[1], seg.color[2], 1.0])[..3]
                .try_into()
                .expect("3ch");
        }
        let bytes = bytemuck::cast_slice(&data);
        let buf = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("strokes"),
            size: bytes.len() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.queue.write_buffer(&buf, 0, bytes);
        self.next_id += 1;
        let id = self.next_id;
        self.strokes_t.insert(
            id,
            GpuInstances {
                buf,
                count: desc.data.len() as u32,
            },
        );
        Ok(id)
    }

    /// Point-table upload (WGPU-18): 32B per entry; empty tables rejected at build (family discipline).
    pub fn create_points(&mut self, desc: &PointTableDesc<'_>) -> Result<TableId, BackendError> {
        if desc.data.is_empty() {
            return Err(BackendError {
                reason: "empty point table".into(),
            });
        }
        let mut data = desc.data.to_vec(); // same shape as chokepoint three of CORE-16 (the point-color linearized copy)
        for m in &mut data {
            m.color = srgb_to_linear([m.color[0], m.color[1], m.color[2], 1.0])[..3]
                .try_into()
                .expect("3ch");
        }
        let bytes = bytemuck::cast_slice(&data);
        let buf = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("points"),
            size: bytes.len() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.queue.write_buffer(&buf, 0, bytes);
        self.next_id += 1;
        let id = self.next_id;
        self.points_t.insert(
            id,
            GpuInstances {
                buf,
                count: desc.data.len() as u32,
            },
        );
        Ok(id)
    }

    /// Point-table destroy (V2.1): GPU buffer + table entry removal; unknown id = silent no-op (idempotent).
    pub fn destroy_points(&mut self, id: TableId) {
        self.points_t.remove(&id);
    }

    /// Label-table upload (WGPU-24): 64B per entry; empty tables rejected at build (family discipline same as points).
    /// Color = the linear domain passed directly (the sRGB chokepoint is on the producer io-text/CAPI face, not here -- the label table
    /// and the point/stroke tables split domains: the latter convert via CPU copies, labels arrive linear-valued from layout upstream).
    pub fn create_labels(&mut self, desc: &LabelTableDesc<'_>) -> Result<TableId, BackendError> {
        if desc.data.is_empty() {
            return Err(BackendError {
                reason: "empty label table".into(),
            });
        }
        let bytes = bytemuck::cast_slice(desc.data);
        let buf = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("labels"),
            size: bytes.len() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.queue.write_buffer(&buf, 0, bytes);
        self.next_id += 1;
        let id = self.next_id;
        self.labels_t.insert(
            id,
            GpuInstances {
                buf,
                count: desc.data.len() as u32,
            },
        );
        Ok(id)
    }

    /// The sum of counts across labels_t tables (the family-semantics assertion entry).
    #[must_use]
    pub fn label_mark_total(&self) -> usize {
        self.labels_t.values().map(|g| g.count as usize).sum()
    }

    /// Glyph atlas upload/rebuild (WGPU-24): R8 single-channel coverage, wholesale replacement (io-text dirty-driven).
    pub fn set_glyph_atlas(
        &mut self,
        r8: &[u8],
        width: u32,
        height: u32,
    ) -> Result<(), BackendError> {
        if r8.len() != (width * height) as usize || width == 0 || height == 0 {
            return Err(BackendError {
                reason: "atlas len != w*h (R8)".into(),
            });
        }
        let tex = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("glyph-atlas"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            r8,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
        self.glyph_atlas = Some((tex, view));
        Ok(())
    }

    fn ensure_sampler(&mut self) {
        if self.sampler.is_none() {
            self.sampler = Some(self.device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("diffuse-samp"),
                address_mode_u: wgpu::AddressMode::Repeat,
                address_mode_v: wgpu::AddressMode::Repeat,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            }));
        }
    }

    #[must_use]
    pub fn supports(capability: Capability) -> bool {
        matches!(capability, Capability::Mesh3D | Capability::OrthoCamera)
    }

    fn ensure_depth(&mut self, width: u32, height: u32) {
        if matches!(&self.depth_cache, Some((w, h, _, _)) if *w == width && *h == height) {
            return;
        }
        let tex = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
        self.depth_cache = Some((width, height, tex, view));
    }

    /// Render one frame to any view (the clear color takes the first ClearColor in commands; DrawMesh redraws fully, v0 has no culling).
    /// The size drives the internal depth face (WGPU-13: occlusion correctness -- near objects hiding far ones does not depend on draw order).
    pub fn render_view(
        &mut self,
        frame: &Frame,
        view: &wgpu::TextureView,
        width: u32,
        height: u32,
    ) {
        self.render_view_format(frame, view, width, height, wgpu::TextureFormat::Rgba8Unorm);
    }

    /// The color-format version (the surface path and the headless fixed format split, the I3 format alignment).
    pub fn render_view_format(
        &mut self,
        frame: &Frame,
        view: &wgpu::TextureView,
        width: u32,
        height: u32,
        color_format: wgpu::TextureFormat,
    ) {
        let _unused: Option<Viewport> = None;
        let clear = frame
            .commands
            .iter()
            .find_map(|c| match c {
                DrawCommand::ClearColor { rgba } => Some(*rgba),
                DrawCommand::DrawMesh { .. }
                | DrawCommand::DrawInstances { .. }
                | DrawCommand::DrawStrokes { .. }
                | DrawCommand::DrawPoints { .. }
                | DrawCommand::DrawLabels { .. } => None,
            })
            .unwrap_or([0.0; 4]);
        // chokepoint two of CORE-16: ClearColor on the host face = sRGB; the srgb target load interprets linearly
        // (probe proof: loading 0.05 raw -> 63 bright), convert to linear first to keep the original CSS-feel byte form
        let cl = srgb_to_linear(clear);
        self.ensure_depth(width.max(1), height.max(1));
        let depth_view = self
            .depth_cache
            .as_ref()
            .map(|(_, _, _, v)| v.clone())
            .expect("depth just ensured");
        let (sh_params, sh_view) = self.shadow_frame_res(frame);
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        // WGPU-33: EDL ON = main pass renders into an internal intermediate color
        // texture (TEXTURE_BINDING-capable), then the EDL fullscreen pass samples
        // intermediate+depth and writes the final view. OFF (None) = legacy direct
        // path untouched (byte-identical; no extra pass, no resource churn).
        // Intermediate (not same-view sampling) because swapchain views lack
        // TEXTURE_BINDING — and wgpu forbids sampling a color attachment of the
        // same encoder section anyway. Surface format must match the pipeline
        // family used for the main pass (per-format pipeline cache covers it).
        let edl_on = frame.edl.is_some();
        // WGPU-34: non-empty post chain = main pass renders into a stage
        // intermediate too (the final view may be a swapchain without
        // TEXTURE_BINDING; every chain pass samples its input). Zero-effect
        // frames never touch this branch (byte-identical legacy path).
        let post_on = !frame.post.is_empty();
        let main_view: wgpu::TextureView;
        let edl_color_view: Option<wgpu::TextureView> = if edl_on {
            let cv = self.ensure_edl_color(width.max(1), height.max(1), color_format);
            main_view = cv.clone();
            Some(cv)
        } else if post_on {
            // Chain input intermediate (stage 0). EDL-off + post-on only.
            let cv = self.ensure_stage_color(0, width.max(1), height.max(1), color_format);
            main_view = cv.clone();
            None
        } else {
            main_view = view.clone();
            None
        };
        self.run_shadow_caster(&mut encoder, frame, color_format, &sh_view);
        self.run_main_pass(
            &mut encoder,
            frame,
            &main_view,
            &depth_view,
            color_format,
            &sh_params,
            &sh_view,
            wgpu::LoadOp::Clear(wgpu::Color {
                r: f64::from(cl[0]),
                g: f64::from(cl[1]),
                b: f64::from(cl[2]),
                a: f64::from(cl[3]),
            }),
            wgpu::LoadOp::Clear(1.0),
            if edl_on || post_on {
                wgpu::StoreOp::Store // post passes consume depth next (EDL + WGPU-34)
            } else {
                wgpu::StoreOp::Discard // OFF = legacy bytes (WGPU-33/34 zero-regression)
            },
            None,
        );
        let edl_out_view = edl_color_view.clone();
        if let (Some(cv), Some(setup)) = (edl_color_view, frame.edl) {
            self.run_edl_pass(
                &mut encoder,
                &cv,
                &depth_view,
                view,
                width.max(1),
                height.max(1),
                setup.strength,
                color_format,
            );
        }
        // WGPU-34: post chain runs AFTER EDL (composes with it); the chain
        // input = EDL output view when EDL ran, else the stage-0 intermediate.
        if post_on {
            debug_assert!(
                frame.post.len() <= 2,
                "post chain >2 effects unsupported (WGPU-34 stages)"
            );
            let chain_input = edl_out_view.unwrap_or_else(|| main_view.clone());
            self.run_post_chain(
                &mut encoder,
                frame,
                &chain_input,
                &depth_view,
                view,
                width.max(1),
                height.max(1),
                color_format,
            );
        }
        self.queue.submit(std::iter::once(encoder.finish()));
    }

    /// WGPU-33: size+format-cached intermediate color target (RENDER_ATTACHMENT |
    /// TEXTURE_BINDING). Rebuilt only on size/format change.
    fn ensure_edl_color(
        &mut self,
        width: u32,
        height: u32,
        color_format: wgpu::TextureFormat,
    ) -> wgpu::TextureView {
        if matches!(
            &self.edl_color,
            Some((w, h, _, f)) if *w == width && *h == height && *f == color_format
        ) {
            let Some((_, _, v, _)) = &self.edl_color else {
                unreachable!("matched above");
            };
            return v.clone();
        }
        let tex = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("edl-color"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: color_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
        self.edl_color = Some((width, height, view.clone(), color_format));
        view
    }

    /// WGPU-33: fullscreen EDL pass — samples intermediate color + main depth,
    /// writes the final view. Depth Load (stored by main pass) + Discard (last
    /// consumer). One draw(0..4) = fullscreen triangle strip per edl.wgsl vs_main.
    #[allow(clippy::too_many_arguments)]
    fn run_edl_pass(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        color_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        out_view: &wgpu::TextureView,
        width: u32,
        height: u32,
        strength: f32,
        color_format: wgpu::TextureFormat,
    ) {
        let pipeline = self.edl_pipeline.entry(color_format).or_insert_with(|| {
            let layout = self
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("edl-pl"),
                    bind_group_layouts: &[Some(&self.edl_bgl)],
                    immediate_size: 0,
                });
            self.device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("edl-pipeline"),
                    layout: Some(&layout),
                    vertex: wgpu::VertexState {
                        module: &self.edl_shader,
                        entry_point: Some("vs_main"),
                        compilation_options: Default::default(),
                        buffers: &[],
                    },
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleStrip,
                        ..Default::default()
                    },
                    depth_stencil: None,
                    multisample: Default::default(),
                    fragment: Some(wgpu::FragmentState {
                        module: &self.edl_shader,
                        entry_point: Some("fs_main"),
                        compilation_options: Default::default(),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: color_format,
                            blend: None,
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    multiview_mask: None,
                    cache: None,
                })
        });
        // params UBO (16B): (texel_w, texel_h, strength, 0). Rebuilt per frame —
        // ponytail: per-frame 64B alloc, switch to write_buffer + cached buf when
        // EDL frames hit a profiled hotspot.
        let params = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("edl-params"),
                contents: bytemuck::cast_slice(&[
                    1.0f32 / width as f32,
                    1.0 / height as f32,
                    strength,
                    0.0,
                ]),
                usage: wgpu::BufferUsages::UNIFORM,
            });
        let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("edl-bg"),
            layout: &self.edl_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(color_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(depth_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.edl_smp),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: params.as_entire_binding(),
                },
            ],
        });
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("edl-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: out_view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            // wgpu 30 usage-scope rule: a depth texture sampled (RESOURCE) cannot
            // simultaneously be a depth-stencil attachment in the same pass —
            // the EDL fragment reads depth via textureLoad only, so the pass
            // drops the depth attachment entirely (main pass already Stored it).
            ..Default::default()
        });
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &bg, &[]);
        pass.draw(0..4, 0..1);
    }

    /// WGPU-34: stage color intermediate, keyed by chain position (0/1). Same
    /// shape as ensure_edl_color (size+format cached, RENDER_ATTACHMENT |
    /// TEXTURE_BINDING). Fixed 2-slot array — chains >2 are a non-goal.
    fn ensure_stage_color(
        &mut self,
        stage: usize,
        width: u32,
        height: u32,
        color_format: wgpu::TextureFormat,
    ) -> wgpu::TextureView {
        assert!(stage < 2, "post chain supports 2 stages max (WGPU-34)");
        if matches!(
            &self.post_stages[stage],
            Some((w, h, _, f)) if *w == width && *h == height && *f == color_format
        ) {
            let Some((_, _, v, _)) = &self.post_stages[stage] else {
                unreachable!("matched above");
            };
            return v.clone();
        }
        let tex = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("post-stage"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: color_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
        self.post_stages[stage] = Some((width, height, view.clone(), color_format));
        view
    }

    /// WGPU-34: one fullscreen post pass — samples color (+depth) in, writes
    /// out. Same pipeline/params/bind-group shape as run_edl_pass; effect
    /// selects shader + params payload. Per-(effect,format) lazy pipeline.
    #[allow(clippy::too_many_arguments)]
    fn run_post_effect_pass(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        effect: usize, // 0 = bloom, 1 = outline, 2 = tonemap
        color_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        out_view: &wgpu::TextureView,
        params: [f32; 4],
        color_format: wgpu::TextureFormat,
    ) {
        let shader = &self.post_shaders[effect];
        let bgl = &self.post_bgl;
        let pipeline = self.post_pipelines[effect]
            .entry(color_format)
            .or_insert_with(|| {
                let layout = self
                    .device
                    .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: Some("post-pl"),
                        bind_group_layouts: &[Some(bgl)],
                        immediate_size: 0,
                    });
                self.device
                    .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                        label: Some("post-pipeline"),
                        layout: Some(&layout),
                        vertex: wgpu::VertexState {
                            module: shader,
                            entry_point: Some("vs_main"),
                            compilation_options: Default::default(),
                            buffers: &[],
                        },
                        primitive: wgpu::PrimitiveState {
                            topology: wgpu::PrimitiveTopology::TriangleStrip,
                            ..Default::default()
                        },
                        depth_stencil: None,
                        multisample: Default::default(),
                        fragment: Some(wgpu::FragmentState {
                            module: shader,
                            entry_point: Some("fs_main"),
                            compilation_options: Default::default(),
                            targets: &[Some(wgpu::ColorTargetState {
                                format: color_format,
                                blend: None,
                                write_mask: wgpu::ColorWrites::ALL,
                            })],
                        }),
                        multiview_mask: None,
                        cache: None,
                    })
            });
        let params_buf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("post-params"),
                contents: bytemuck::cast_slice(&params),
                usage: wgpu::BufferUsages::UNIFORM,
            });
        let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("post-bg"),
            layout: &self.post_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(color_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(depth_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.edl_smp),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: params_buf.as_entire_binding(),
                },
            ],
        });
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("post-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: out_view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            // Same usage-scope stance as EDL (WGPU-33): depth sampled via
            // textureLoad only; no depth attachment in this pass.
            ..Default::default()
        });
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &bg, &[]);
        pass.draw(0..4, 0..1);
    }

    /// WGPU-34: chain `frame.post` effects over the main-pass output. Input =
    /// EDL output view when EDL ran, else the direct target view. Each effect
    /// stages through an intermediate EXCEPT the last, which writes the final
    /// view (final view is write-only — wgpu usage-scope law). params payload:
    /// (texel_w, texel_h, strength/width-mix, width-texels).
    #[allow(clippy::too_many_arguments)]
    fn run_post_chain(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        frame: &Frame,
        chain_input: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        final_view: &wgpu::TextureView,
        width: u32,
        height: u32,
        color_format: wgpu::TextureFormat,
    ) {
        let n = frame.post.len();
        let mut current = chain_input.clone();
        for (i, effect) in frame.post.iter().enumerate() {
            // Effect i output: last effect -> final view; otherwise stage i+1
            // (stage 0 is the chain INPUT intermediate — never a write target,
            // else the input pass would conflict RESOURCE+COLOR_TARGET).
            let out_view: wgpu::TextureView = if i + 1 == n {
                final_view.clone()
            } else {
                self.ensure_stage_color(i + 1, width, height, color_format)
            };
            let (strength, width_px) = match *effect {
                visiaengine_render::PostEffect::Bloom { strength } => (strength, 0.0),
                visiaengine_render::PostEffect::Outline { width } => (width, width),
                visiaengine_render::PostEffect::Tonemap { .. } => (0.0, 0.0),
                visiaengine_render::PostEffect::Ssao { .. } => (0.0, 0.0),
                visiaengine_render::PostEffect::Haze { .. } => (0.0, 0.0),
            };
            let effect_idx = match effect {
                visiaengine_render::PostEffect::Bloom { .. } => 0usize,
                visiaengine_render::PostEffect::Outline { .. } => 1usize,
                visiaengine_render::PostEffect::Tonemap { .. } => 2usize,
                visiaengine_render::PostEffect::Ssao { .. } => 3usize,
                visiaengine_render::PostEffect::Haze { .. } => 4usize,
            };
            let params: [f32; 4] = match effect {
                visiaengine_render::PostEffect::Bloom { .. } => {
                    [1.0 / width as f32, 1.0 / height as f32, strength, 0.0]
                }
                visiaengine_render::PostEffect::Outline { .. } => [
                    1.0 / width as f32,
                    1.0 / height as f32,
                    strength,
                    width_px.round().max(1.0),
                ],
                visiaengine_render::PostEffect::Tonemap { mode } => [*mode as f32, 0.0, 0.0, 0.0],
                visiaengine_render::PostEffect::Ssao { radius, intensity } => {
                    // WGPU-36 params packing: (texel_w, texel_h, radius, intensity).
                    [1.0 / width as f32, 1.0 / height as f32, *radius, *intensity]
                }
                visiaengine_render::PostEffect::Haze {
                    far_start,
                    far_full,
                } => {
                    // WGPU-38 params packing: (texel_w, texel_h, far_start, far_full).
                    [
                        1.0 / width as f32,
                        1.0 / height as f32,
                        *far_start,
                        *far_full,
                    ]
                }
            };
            self.run_post_effect_pass(
                encoder,
                effect_idx,
                &current,
                depth_view,
                &out_view,
                params,
                color_format,
            );
            current = out_view;
        }
    }

    /// 5b multi-viewport splitting (WGPU-26): N scissor-locked regions on one surface / one full-frame depth,
    /// each carrying its own Frame (cameras independent, commands shared). **The safe form = first pass Clear (full-frame background = seam color)
    /// + subsequent Load**; `AllClear` serves only as the ruling probe. depth: non-last passes Store, the last pass Discard
    ///   (the same old form); one caster run (frame0 configuration -- light-space is independent of the main camera, both projections share [ruling d]).
    ///   full-rect sets no viewport/scissor = bitwise equal to the old path (canary lock).
    pub fn render_view_rects(
        &mut self,
        passes: &[(Frame, visiaengine_render::ViewportRect)],
        view: &wgpu::TextureView,
        width: u32,
        height: u32,
        color_format: wgpu::TextureFormat,
        policy: MultiClearPolicy,
    ) {
        let Some((head_frame, _)) = passes.first() else {
            return;
        };
        // Review finding #7: EDL is single-view-only this band (WGPU-33
        // clause); a Some(edl) frame here would be SILENTLY ignored. Make
        // the misuse loud in debug builds.
        debug_assert!(
            passes.iter().all(|(f, _)| f.edl.is_none()),
            "render_view_rects ignores Frame.edl (multiview + EDL unsupported; WGPU-33)"
        );
        debug_assert!(
            passes.iter().all(|(f, _)| f.post.is_empty()),
            "render_view_rects ignores Frame.post (multiview + post chain unsupported; WGPU-34)"
        );
        let clear = head_frame
            .commands
            .iter()
            .find_map(|c| match c {
                DrawCommand::ClearColor { rgba } => Some(*rgba),
                _ => None,
            })
            .unwrap_or([0.0; 4]);
        let cl = srgb_to_linear(clear);
        self.ensure_depth(width.max(1), height.max(1));
        let depth_view = self
            .depth_cache
            .as_ref()
            .map(|(_, _, _, v)| v.clone())
            .expect("depth just ensured");
        let (_sh0_params, sh0_view) = self.shadow_frame_res(head_frame);
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        self.run_shadow_caster(&mut encoder, head_frame, color_format, &sh0_view);
        for (i, (frame, rect)) in passes.iter().enumerate() {
            let (sfp, sfv) = self.shadow_frame_res(frame);
            let clear_this = i == 0 || matches!(policy, MultiClearPolicy::AllClear);
            let full = *rect == visiaengine_render::ViewportRect::full(width, height);
            self.run_main_pass(
                &mut encoder,
                frame,
                view,
                &depth_view,
                color_format,
                &sfp,
                &sfv,
                if clear_this {
                    wgpu::LoadOp::Clear(wgpu::Color {
                        r: f64::from(cl[0]),
                        g: f64::from(cl[1]),
                        b: f64::from(cl[2]),
                        a: f64::from(cl[3]),
                    })
                } else {
                    wgpu::LoadOp::Load
                },
                // WGPU-26 settled correction: depth **must clear per pass** (the first pass's full-frame clear covers the whole region;
                // if subsequent passes Load, they take the main view's opaque-ground depth as occlusion = the minimap is swallowed whole,
                // caught live by the 2026-09-18 pixel gate); color clears only on the first pass (= seam background), later regions are not wiped.
                // No depth consumer between passes => all Discard (saves Store bandwidth).
                wgpu::LoadOp::Clear(1.0),
                wgpu::StoreOp::Discard,
                if full { None } else { Some(*rect) },
            );
        }
        self.queue.submit(std::iter::once(encoder.finish()));
    }

    fn run_shadow_caster(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        frame: &Frame,
        color_format: wgpu::TextureFormat,
        sh_view: &wgpu::TextureView,
    ) {
        // ===== WGPU-19 caster pre-pass (runs only when shadow = Some; DrawMesh/DrawInstances project,
        // the tile-expansion families do not project [not-installed 2]; a pass with no color target, bias lives in the caster pipeline's depth_stencil) =====
        if frame.shadow.is_some() && self.shadow_active(frame) {
            let mut sp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("shadow-pass"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: sh_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });
            for cmd in &frame.commands {
                let (mesh, origin, transform, inst_id) = match *cmd {
                    DrawCommand::DrawMesh {
                        mesh,
                        origin,
                        transform,
                        ..
                    } => (mesh, origin, transform, None),
                    DrawCommand::DrawInstances {
                        mesh,
                        origin,
                        transform,
                        instances,
                        ..
                    } => (mesh, origin, transform, Some(instances)),
                    _ => continue,
                };
                let vblock = self.uniform(
                    bytemuck::cast_slice(&self.view_block(frame, &origin, &transform)),
                    "view-shadow",
                );
                let Some(gm) = self.meshes.get(&mesh) else {
                    continue;
                };
                let (vbuf, ibuf, index_count) = (gm.vbuf.clone(), gm.ibuf.clone(), gm.index_count);
                let variant = if inst_id.is_some() {
                    Variant::ShadowInst
                } else {
                    Variant::ShadowMesh
                };
                let pipeline = match self.pipelines.get(&(color_format, variant)) {
                    Some(p) => p.clone(),
                    None => {
                        let layout = if inst_id.is_some() {
                            &self.layout_shadow_inst
                        } else {
                            &self.layout_shadow
                        };
                        let p = build_pipeline(
                            &self.device,
                            layout,
                            &self.shader,
                            color_format,
                            variant,
                        );
                        self.pipelines.insert((color_format, variant), p.clone());
                        p
                    }
                };
                sp.set_pipeline(&pipeline);
                let mut inst_buf: Option<wgpu::Buffer> = None;
                let mut inst_count = 1u32;
                if let Some(iid) = inst_id {
                    let Some(gi) = self.instances.get(&iid) else {
                        continue;
                    };
                    inst_buf = Some(gi.buf.clone());
                    inst_count = gi.count;
                }
                let mut entries = vec![wgpu::BindGroupEntry {
                    binding: 0,
                    resource: vblock.as_entire_binding(),
                }];
                if let Some(ib) = &inst_buf {
                    entries.push(wgpu::BindGroupEntry {
                        binding: 5,
                        resource: ib.as_entire_binding(),
                    });
                }
                let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("shadow-caster-bg"),
                    layout: if inst_id.is_some() {
                        &self.bgl_shadow_inst
                    } else {
                        &self.bgl_shadow
                    },
                    entries: &entries,
                });
                sp.set_bind_group(0, &bg, &[]);
                sp.set_vertex_buffer(0, vbuf.slice(..));
                sp.set_index_buffer(ibuf.slice(..), wgpu::IndexFormat::Uint32);
                sp.draw_indexed(0..index_count, 0, 0..inst_count);
            }
        }
    }

    /// Two-pass orchestration (4) [WGPU-28]: the transparent family = sorted by material a < 1 (REND-36 single decision) ->
    /// pass1 opaque (vec order as-is) + pass2 transparent sorted by view depth descending (painter, D7 f64 distances) + Labels always last on top;
    /// **without transparency and without labels = the old single-pass path (byte preserved)**; pass2 depth Load inherits pass1 depth [Momus-A2].
    #[allow(clippy::too_many_arguments, clippy::too_many_lines)]
    fn run_main_pass(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        frame: &Frame,
        view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        color_format: wgpu::TextureFormat,
        sh_params: &wgpu::Buffer,
        sh_view: &wgpu::TextureView,
        color_load: wgpu::LoadOp<wgpu::Color>,
        depth_load: wgpu::LoadOp<f32>,
        depth_store: wgpu::StoreOp,
        rect: Option<visiaengine_render::ViewportRect>,
    ) {
        let mut trans: Vec<&DrawCommand> = Vec::new();
        let mut opaque: Vec<&DrawCommand> = Vec::new();
        let mut labels: Vec<&DrawCommand> = Vec::new();
        for cmd in &frame.commands {
            match cmd {
                DrawCommand::DrawMesh { material, .. }
                | DrawCommand::DrawInstances { material, .. }
                    if self.materials.get(material).is_some_and(|m| m.alpha < 1.0) =>
                {
                    trans.push(cmd);
                }
                DrawCommand::DrawLabels { .. } => labels.push(cmd),
                _ => opaque.push(cmd),
            }
        }
        if trans.is_empty() && labels.is_empty() {
            let all: Vec<&DrawCommand> = frame.commands.iter().collect();
            self.one_main_pass(
                encoder,
                frame,
                &all,
                view,
                depth_view,
                color_format,
                sh_params,
                sh_view,
                color_load,
                depth_load,
                depth_store,
                rect,
            );
            return;
        }
        let p1_store = if trans.is_empty() {
            depth_store
        } else {
            wgpu::StoreOp::Store // the transparent pass must carry the opaque depth [Momus-A2]
        };
        self.one_main_pass(
            encoder,
            frame,
            &opaque,
            view,
            depth_view,
            color_format,
            sh_params,
            sh_view,
            color_load,
            depth_load,
            p1_store,
            rect,
        );
        if !trans.is_empty() {
            let eye = frame.eye;
            trans.sort_by(|a, b| cmd_view_depth2(b, eye).total_cmp(&cmd_view_depth2(a, eye)));
            trans.extend(labels.iter().copied());
            self.one_main_pass(
                encoder,
                frame,
                &trans,
                view,
                depth_view,
                color_format,
                sh_params,
                sh_view,
                wgpu::LoadOp::Load,
                wgpu::LoadOp::Load,
                wgpu::StoreOp::Discard,
                rect,
            );
        } else {
            self.one_main_pass(
                encoder,
                frame,
                &labels,
                view,
                depth_view,
                color_format,
                sh_params,
                sh_view,
                wgpu::LoadOp::Load,
                wgpu::LoadOp::Clear(1.0),
                wgpu::StoreOp::Discard,
                rect,
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn one_main_pass(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        frame: &Frame,
        cmds: &[&DrawCommand],
        view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        color_format: wgpu::TextureFormat,
        sh_params: &wgpu::Buffer,
        sh_view: &wgpu::TextureView,
        color_load: wgpu::LoadOp<wgpu::Color>,
        depth_load: wgpu::LoadOp<f32>,
        depth_store: wgpu::StoreOp,
        rect: Option<visiaengine_render::ViewportRect>,
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("mesh-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: color_load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: depth_load,
                    store: depth_store,
                }),
                stencil_ops: None,
            }),
            ..Default::default()
        });
        // scissor-locked raster (the color/depth write fence); full-rect does not take this branch = bitwise equal to the old path (canary)
        if let Some(r) = rect {
            pass.set_scissor_rect(r.x, r.y, r.width.max(1), r.height.max(1));
            pass.set_viewport(
                r.x as f32,
                r.y as f32,
                r.width as f32, // the scissor/viewport domain (sizes stay small values, the cast is safe)
                r.height as f32,
                0.0,
                1.0,
            );
        }
        self.draw_frame_commands(&mut pass, frame, cmds, color_format, sh_params, sh_view);
    }

    fn draw_frame_commands(
        &mut self,
        pass: &mut wgpu::RenderPass,
        frame: &Frame,
        cmds: &[&DrawCommand],
        color_format: wgpu::TextureFormat,
        sh_params: &wgpu::Buffer,
        sh_view: &wgpu::TextureView,
    ) {
        for &cmd in cmds {
            match cmd {
                // N1 half-road: the tile-expansion family arms take placeholders first (N2/N3 implement the imagery) -- the exhaustive synchronizer locks here,
                // a missing arm = compile failure (the contract face delivered, not a silent skip)
                DrawCommand::ClearColor { .. } => {}
                DrawCommand::DrawLabels {
                    table,
                    origin,
                    transform,
                } => {
                    let mvp_buf = self.uniform(
                        bytemuck::cast_slice(&self.view_block(frame, origin, transform)),
                        "view",
                    );
                    let Some(gi) = self.labels_t.get(table) else {
                        continue; // missing table = skip (family discipline, same scheme as WGPU-18)
                    };
                    let Some((_, atlas_view)) = &self.glyph_atlas else {
                        static ONCE: std::sync::Once = std::sync::Once::new();
                        ONCE.call_once(|| {
                            eprintln!("warn: labels present but atlas unset — skipped [WGPU-24]");
                        });
                        continue; // atlas not uploaded = the whole family invisible (family discipline + warn-once)
                    };
                    let tbuf = gi.buf.clone();
                    let count = gi.count;
                    let apv = atlas_view.clone();
                    let pipeline = match self.pipelines.get(&(color_format, Variant::Labels)) {
                        Some(p) => p.clone(),
                        None => {
                            let p = build_pipeline(
                                &self.device,
                                &self.layout_label,
                                &self.shader,
                                color_format,
                                Variant::Labels,
                            );
                            self.pipelines
                                .insert((color_format, Variant::Labels), p.clone());
                            p
                        }
                    };
                    pass.set_pipeline(&pipeline);
                    let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("label-bg"),
                        layout: &self.bgl_label,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: mvp_buf.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 11,
                                resource: tbuf.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 12,
                                resource: wgpu::BindingResource::TextureView(&apv),
                            },
                            wgpu::BindGroupEntry {
                                binding: 13,
                                resource: wgpu::BindingResource::Sampler(&self.label_smp),
                            },
                        ],
                    });
                    pass.set_bind_group(0, &bg, &[]);
                    pass.set_vertex_buffer(0, self.quad_vb.slice(..));
                    pass.set_index_buffer(self.quad_ib.slice(..), wgpu::IndexFormat::Uint32);
                    pass.draw_indexed(0..6, 0, 0..count);
                }
                DrawCommand::DrawPoints {
                    table,
                    origin,
                    transform,
                } => {
                    let mvp_buf = self.uniform(
                        bytemuck::cast_slice(&self.view_block(frame, origin, transform)),
                        "view",
                    );
                    let Some(gi) = self.points_t.get(table) else {
                        continue; // missing table = skip (family discipline, WGPU-18)
                    };
                    let tbuf = gi.buf.clone();
                    let count = gi.count;
                    let pipeline = match self.pipelines.get(&(color_format, Variant::Points)) {
                        Some(p) => p.clone(),
                        None => {
                            let p = build_pipeline(
                                &self.device,
                                &self.layout_point,
                                &self.shader,
                                color_format,
                                Variant::Points,
                            );
                            self.pipelines
                                .insert((color_format, Variant::Points), p.clone());
                            p
                        }
                    };
                    pass.set_pipeline(&pipeline);
                    let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("point-bg"),
                        layout: &self.bgl_point,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: mvp_buf.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 6,
                                resource: tbuf.as_entire_binding(),
                            },
                        ],
                    });
                    pass.set_bind_group(0, &bg, &[]);
                    pass.set_vertex_buffer(0, self.quad_vb.slice(..));
                    pass.set_index_buffer(self.quad_ib.slice(..), wgpu::IndexFormat::Uint32);
                    pass.draw_indexed(0..6, 0, 0..count);
                }
                DrawCommand::DrawMesh {
                    mesh,
                    material,
                    origin,
                    transform,
                } => {
                    let mvp_buf = self.uniform(
                        bytemuck::cast_slice(&self.view_block(frame, origin, transform)),
                        "view",
                    );
                    let (vbuf, ibuf, index_count) = match self.meshes.get(mesh) {
                        Some(gm) => (gm.vbuf.clone(), gm.ibuf.clone(), gm.index_count),
                        None => continue,
                    };
                    let Some(mat) = self.materials.get(material) else {
                        continue;
                    };
                    let buf = mat.buf.clone();
                    let (textured, view) = (mat.textured, mat.view.clone());
                    let transparent = mat.alpha < 1.0;
                    let variant = match (textured, transparent) {
                        (false, false) => Variant::Flat,
                        (false, true) => Variant::FlatT,
                        (true, false) => Variant::Textured,
                        (true, true) => Variant::TexturedT,
                    };
                    let pipeline = match self.pipelines.get(&(color_format, variant)) {
                        Some(p) => p.clone(),
                        None => {
                            let layout = if textured {
                                &self.layout_tex
                            } else {
                                &self.layout
                            };
                            let p = build_pipeline(
                                &self.device,
                                layout,
                                &self.shader,
                                color_format,
                                variant,
                            );
                            self.pipelines.insert((color_format, variant), p.clone());
                            p
                        }
                    };
                    pass.set_pipeline(&pipeline);
                    let bg = if textured {
                        let Some(view) = view else { continue };
                        self.ensure_sampler();
                        let sampler = self.sampler.clone().expect("sampler just ensured");
                        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                            label: Some("mesh-bg-tex"),
                            layout: &self.bgl_tex,
                            entries: &[
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: mvp_buf.as_entire_binding(),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 2,
                                    resource: buf.as_entire_binding(),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 3,
                                    resource: wgpu::BindingResource::TextureView(&view),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 4,
                                    resource: wgpu::BindingResource::Sampler(&sampler),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 1,
                                    resource: sh_params.as_entire_binding(),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 7,
                                    resource: wgpu::BindingResource::TextureView(sh_view),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 8,
                                    resource: wgpu::BindingResource::Sampler(&self.sh_sampler),
                                },
                            ],
                        })
                    } else {
                        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                            label: Some("mesh-bg"),
                            layout: &self.bgl,
                            entries: &[
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: mvp_buf.as_entire_binding(),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 2,
                                    resource: buf.as_entire_binding(),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 1,
                                    resource: sh_params.as_entire_binding(),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 7,
                                    resource: wgpu::BindingResource::TextureView(sh_view),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 8,
                                    resource: wgpu::BindingResource::Sampler(&self.sh_sampler),
                                },
                            ],
                        })
                    };
                    pass.set_bind_group(0, &bg, &[]);
                    pass.set_vertex_buffer(0, vbuf.slice(..));
                    pass.set_index_buffer(ibuf.slice(..), wgpu::IndexFormat::Uint32);
                    pass.draw_indexed(0..index_count, 0, 0..1);
                }
                // ponytail: the two arms' ~25 lines of mvp/table-lookup/pipeline-cache share shape -- the trigger to
                // extract a shared helper = the 4de third consumer (line/point tile-expansion); for now the Flat/Textured paths stay verbatim = the zero-regression construction guarantee
                DrawCommand::DrawInstances {
                    mesh,
                    material,
                    instances,
                    origin,
                    transform,
                } => {
                    let mvp_buf = self.uniform(
                        bytemuck::cast_slice(&self.view_block(frame, origin, transform)),
                        "view",
                    );
                    let (vbuf, ibuf, index_count) = match self.meshes.get(mesh) {
                        Some(gm) => (gm.vbuf.clone(), gm.ibuf.clone(), gm.index_count),
                        None => continue,
                    };
                    let Some(gi) = self.instances.get(instances) else {
                        continue; // missing table = skip (the same discipline as missing meshes, WGPU-16)
                    };
                    let inst_buf = gi.buf.clone();
                    let count = gi.count;
                    let Some(mat) = self.materials.get(material) else {
                        continue;
                    };
                    let buf = mat.buf.clone();
                    let variant = if mat.alpha < 1.0 {
                        Variant::InstancedT
                    } else {
                        Variant::Instanced
                    };
                    let pipeline = match self.pipelines.get(&(color_format, variant)) {
                        Some(p) => p.clone(),
                        None => {
                            let p = build_pipeline(
                                &self.device,
                                &self.layout_inst,
                                &self.shader,
                                color_format,
                                variant,
                            );
                            self.pipelines.insert((color_format, variant), p.clone());
                            p
                        }
                    };
                    pass.set_pipeline(&pipeline);
                    let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("mesh-bg-inst"),
                        layout: &self.bgl_inst,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: mvp_buf.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 2,
                                resource: buf.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 5,
                                resource: inst_buf.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: sh_params.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 7,
                                resource: wgpu::BindingResource::TextureView(sh_view),
                            },
                            wgpu::BindGroupEntry {
                                binding: 8,
                                resource: wgpu::BindingResource::Sampler(&self.sh_sampler),
                            },
                        ],
                    });
                    pass.set_bind_group(0, &bg, &[]);
                    pass.set_vertex_buffer(0, vbuf.slice(..));
                    pass.set_index_buffer(ibuf.slice(..), wgpu::IndexFormat::Uint32);
                    pass.draw_indexed(0..index_count, 0, 0..count);
                }
                DrawCommand::DrawStrokes {
                    table,
                    origin,
                    transform,
                } => {
                    let mvp_buf = self.uniform(
                        bytemuck::cast_slice(&self.view_block(frame, origin, transform)),
                        "view",
                    );
                    let Some(gi) = self.strokes_t.get(table) else {
                        continue; // missing table = skip (family discipline, WGPU-17)
                    };
                    let tbuf = gi.buf.clone();
                    let count = gi.count;
                    let pipeline = match self.pipelines.get(&(color_format, Variant::Strokes)) {
                        Some(p) => p.clone(),
                        None => {
                            let p = build_pipeline(
                                &self.device,
                                &self.layout_stroke,
                                &self.shader,
                                color_format,
                                Variant::Strokes,
                            );
                            self.pipelines
                                .insert((color_format, Variant::Strokes), p.clone());
                            p
                        }
                    };
                    pass.set_pipeline(&pipeline);
                    let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("stroke-bg"),
                        layout: &self.bgl_stroke,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: mvp_buf.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 5,
                                resource: tbuf.as_entire_binding(),
                            },
                        ],
                    });
                    pass.set_bind_group(0, &bg, &[]);
                    pass.set_vertex_buffer(0, self.quad_vb.slice(..));
                    pass.set_index_buffer(self.quad_ib.slice(..), wgpu::IndexFormat::Uint32);
                    pass.draw_indexed(0..6, 0, 0..count);
                }
            }
        }
    }
}

/// The painter-sort key: |.-eye|^2 of the entity face center ~ origin + the M translation column (the f64 D7 domain;
/// far-origin subtraction follows the same law, PIT-5/REND-20). Equal keys keep submission order stable (the sort_by stability is a construction guarantee [ruling h]).
fn cmd_view_depth2(cmd: &DrawCommand, eye: [f64; 3]) -> f64 {
    let (o, t) = match *cmd {
        DrawCommand::DrawMesh {
            origin, transform, ..
        }
        | DrawCommand::DrawInstances {
            origin, transform, ..
        } => (origin, transform),
        _ => return 0.0,
    };
    (0..3).fold(0.0f64, |acc, i| {
        let d = o[i] + t[3][i] - eye[i];
        acc + d * d
    })
}

/// Build the pipeline by the target color format (`Rgba8Unorm` vs the `Bgra8Unorm*` family is decided by the surface caps).
#[must_use]
#[allow(clippy::cast_precision_loss, clippy::too_many_lines)]
fn build_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    color_format: wgpu::TextureFormat,
    variant: Variant,
) -> wgpu::RenderPipeline {
    let stroke_attrs = wgpu::vertex_attr_array![0 => Float32x2];
    let mesh_attrs = wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x2];
    let buffers: [Option<wgpu::VertexBufferLayout>; 1] = if matches!(
        variant,
        Variant::Strokes | Variant::Points | Variant::Labels
    ) {
        [Some(wgpu::VertexBufferLayout {
            array_stride: (2 * std::mem::size_of::<f32>()) as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &stroke_attrs,
        })]
    } else {
        [Some(wgpu::VertexBufferLayout {
            array_stride: (8 * std::mem::size_of::<f32>()) as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &mesh_attrs,
        })]
    };
    let cts = [Some(wgpu::ColorTargetState {
        format: color_format,
        blend: None,
        write_mask: wgpu::ColorWrites::ALL,
    })];
    // WGPU-24: labels = alpha blending (coverage emitted directly) -- the other families stay unblended opaque
    let cts_label = [Some(wgpu::ColorTargetState {
        format: color_format,
        blend: Some(wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
        }),
        write_mask: wgpu::ColorWrites::ALL,
    })];
    let color_targets: &[Option<wgpu::ColorTargetState>] = match variant {
        Variant::ShadowMesh | Variant::ShadowInst => &[],
        // The transparent family (4) = Labels with the same standard form (Momus note: this form is ALPHA_BLENDING, not premultiplied standard)
        Variant::Labels | Variant::FlatT | Variant::TexturedT | Variant::InstancedT => &cts_label,
        _ => &cts,
    };
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("mesh-pipeline"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some(match variant {
                Variant::Instanced | Variant::InstancedT => "vs_inst",
                Variant::Strokes => "vs_stroke",
                Variant::Points => "vs_point",
                Variant::Labels => "vs_label",
                Variant::ShadowMesh => "vs_shadow",
                Variant::ShadowInst => "vs_shadow_inst",
                _ => "vs",
            }),
            compilation_options: Default::default(),
            buffers: &buffers,
        },
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: DEPTH_FORMAT,
            // Labels always on top (WGPU-24) [ruling point e]: labels write no depth + Always (coverage semantics; the command order places them last naturally)
            // The transparent family (WGPU-27): no depth writes (mutual visibility; [Momus-A2] ordering handled by the passes)
            depth_write_enabled: Some(!matches!(
                variant,
                Variant::Labels | Variant::FlatT | Variant::TexturedT | Variant::InstancedT
            )),
            depth_compare: Some(if matches!(variant, Variant::Labels) {
                wgpu::CompareFunction::Always
            } else {
                wgpu::CompareFunction::Less
            }),
            stencil: wgpu::StencilState::default(),
            // polygon-offset companion piece [E3D:B7-3]: the tile-expansion family caps against it without fighting same-depth z-fight
            bias: if matches!(variant, Variant::Strokes | Variant::Points) {
                wgpu::DepthBiasState {
                    constant: -1,
                    slope_scale: -1.0,
                    ..Default::default()
                }
            } else if matches!(variant, Variant::ShadowMesh | Variant::ShadowInst) {
                // double insurance against acne on the caster side (the receiver carries a constant bias as well [R2]; keep in sync with DEFAULT_BIAS)
                wgpu::DepthBiasState {
                    constant: -1,
                    slope_scale: -0.5,
                    ..Default::default()
                }
            } else {
                wgpu::DepthBiasState::default()
            },
        }),
        multisample: Default::default(),
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some(match variant {
                Variant::Textured | Variant::TexturedT => "fs_textured",
                Variant::Strokes => "fs_stroke",
                Variant::Points => "fs_point",
                Variant::Labels => "fs_label",
                Variant::ShadowMesh | Variant::ShadowInst => "fs_shadow",
                _ => "fs",
            }),
            compilation_options: Default::default(),
            targets: color_targets,
        }),
        multiview_mask: None,
        cache: None,
    })
}
