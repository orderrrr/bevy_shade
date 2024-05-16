<script>
  import init from "/shade/bevy_shade.js?url";
  import { push_shader } from "/js_reader.js?url";
  import { onMount } from "svelte";
  import CodeMirror from "svelte-codemirror-editor";
  import { oneDark } from "@codemirror/theme-one-dark";
  import { wgsl } from "@iizukak/codemirror-lang-wgsl";

  let compute_value = 
`// This shader computes the chromatic aberration effect
#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_render::globals::Globals

struct Voxel {
    col: u32,
    mat: u32,
}

pub struct OCTree {
    offset: u32,
    mask: u32,
}

@group(1) @binding(0) var<storage, read_write> particle_buffer : OCTree;
@group(1) @binding(1) var<storage, read_write> indirect_buffer : Voxel;

@compute @workgroup_size(16, 1, 1)
fn compute(@builtin(global_invocation_id) invocation_id: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>) -> Output {
}
`;

  let value = 
`// This shader computes the chromatic aberration effect
#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_render::globals::Globals

struct Object {
    
    mat: f32,
    typ: f32,
    pos: vec3<f32>,
    dat00: vec4<f32>,
    dat01: vec4<f32>,
}

struct OCTree {

    posi: vec3<f32>,
    scle: vec3<f32>,
    chnk: array<vec2<i32>, 8>,
    data: array<vec2<i32>>,
}

@group(0) @binding(0) var<uniform> globals: Globals;
@group(0) @binding(1) var<uniform> octree: array<OCTree>; // idx 00 is scene start
@group(0) @binding(2) var<uniform> data: array<Object>;

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

    var out: Output;
    out.history = vec4(col);
    out.view_target = vec4(col);
    return out;
}
`;

  push_shader("shaders/compute.wgsl", compute_value);
  push_shader("shaders/fragment.wgsl", value);

  // so does work.
  // push_shader("shaders/fragment.wgsl", "BROKEN");

  onMount(() => {
    init().catch((error) => {
      if (
        !error.message.startsWith(
          "Using exceptions for control flow, don't mind me. This isn't actually an error!",
        )
      ) {
        throw error;
      }
    });
  });

  const handle_change = () => {

    push_shader("shaders/fragment.wgsl", value);
  };
</script>

<CodeMirror class="text_edit" bind:value lang={wgsl()} theme={oneDark} on:change={handle_change} />

<style>
  .text_edit {
    position: absolute;
  }
</style>

