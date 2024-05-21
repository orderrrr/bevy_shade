#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_render::globals::Globals

struct OCTree {
    @location(0) offset: u32,
    @location(1) mask: u32,
}

struct Voxel {
    @location(0) col: u32,
    @location(1) mat: u32,
}

@group(0) @binding(0) var<uniform> globals: Globals;
@group(0) @binding(1) var<storage, read> octrees: array<OCTree>;
@group(0) @binding(2) var<storage, read> voxels: array<Voxel>;

@group(0) @binding(3) var screen_texture: texture_2d<f32>;
@group(0) @binding(4) var prev_frame: texture_2d<f32>;

@group(0) @binding(5) var nearest_sampler: sampler;
@group(0) @binding(6) var linear_sampler: sampler;

struct Output {
  @location(0) view_target: vec4<f32>,
  @location(1) history: vec4<f32>,
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> Output {
 // Chromatic aberration strength
    let offset_strength = 0.1;

 // Sample each color channel with an arbitrary shift
    var col = vec4<f32>(
        textureSample(screen_texture, nearest_sampler, in.uv + vec2<f32>(offset_strength, -offset_strength)).r,
        textureSample(screen_texture, nearest_sampler, in.uv + vec2<f32>(-offset_strength, 0.0)).g,
        textureSample(screen_texture, nearest_sampler, in.uv + vec2<f32>(0.0, offset_strength)).b,
        1.0
    );

    col = vec4(1.0);

    if (octrees[8].mask > 0) {
        col = vec4(0., 1., 1., 1.);
    }

    var mask = 0u;

    mask = insertBits(mask, 1u, 0u, 1u);

    // if (mask >= 1u) {
    //     col = vec4(0., 1., 1., 1.);
    // }

    var out: Output;
    out.history = vec4(col);
    out.view_target = vec4(col);
    return out;
}
