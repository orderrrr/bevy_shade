const RESOURCES = {
  "shaders/fragment.wgsl":
    "// This shader computes the chromatic aberration effect\n\n// Since post processing is a fullscreen effect, we use the fullscreen vertex shader provided by bevy.\n// This will import a vertex shader that renders a single fullscreen triangle.\n//\n// A fullscreen triangle is a single triangle that covers the entire screen.\n// The box in the top left in that diagram is the screen. The 4 x are the corner of the screen\n//\n// Y axis\n//  1 |  x-----x......\n//  0 |  |  s  |  . ´\n// -1 |  x_____x´\n// -2 |  :  .´\n// -3 |  :´\n//    +---------------  X axis\n//      -1  0  1  2  3\n//\n// As you can see, the triangle ends up bigger than the screen.\n//\n// You don't need to worry about this too much since bevy will compute the correct UVs for you.\n#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput\n\n@group(0) @binding(0) var screen_texture: texture_2d<f32>;\n@group(0) @binding(1) var texture_sampler: sampler;\nstruct PostProcessSettings {\n    intensity: f32,\n#ifdef SIXTEEN_BYTE_ALIGNMENT\n    // WebGL2 structs must be 16 byte aligned.\n    _webgl2_padding: vec3<f32>\n#endif\n}\n@group(0) @binding(2) var<uniform> settings: PostProcessSettings;\n\n@fragment\nfn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {\n    // Chromatic aberration strength\n    let offset_strength = settings.intensity;\n\n    // Sample each color channel with an arbitrary shift\n    return vec4<f32>(\n        textureSample(screen_texture, texture_sampler, in.uv + vec2<f32>(offset_strength, -offset_strength)).r,\n        textureSample(screen_texture, texture_sampler, in.uv + vec2<f32>(-offset_strength, 0.0)).g,\n        textureSample(screen_texture, texture_sampler, in.uv + vec2<f32>(0.0, offset_strength)).b,\n        1.0\n    );\n}",
};

export const fetch_shader = (path) => {
  return RESOURCES[path];
};

export const push_shader = (path, data) => {
  RESOURCES[path] = data;
};