// This shader computes the chromatic aberration effect
#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var prev_frame: texture_2d<f32>;

@group(0) @binding(2) var nearest_sampler: sampler;
@group(0) @binding(3) var linear_sampler: sampler;

struct Globals {
 resolution: vec2<f32>,
 time: f32,
}
@group(0) @binding(4) var<uniform> globals: Globals;

struct Output {
  @location(0) view_target: vec4<f32>,
  @location(1) history: vec4<f32>,
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> Output {
 // Chromatic aberration strength
 let offset_strength = 0.1;

 // Sample each color channel with an arbitrary shift
 var col =  vec4<f32>(
     textureSample(screen_texture, nearest_sampler, in.uv + vec2<f32>(offset_strength, -offset_strength)).r,
     textureSample(screen_texture, nearest_sampler, in.uv + vec2<f32>(-offset_strength, 0.0)).g,
     textureSample(screen_texture, nearest_sampler, in.uv + vec2<f32>(0.0, offset_strength)).b,
     1.0
 );

  var out: Output;
  out.history = vec4(col);
  out.view_target = vec4(col);
  return out;
}
