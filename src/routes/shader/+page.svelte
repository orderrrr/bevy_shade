<script>
    import init from "/shade/bevy_shade.js?url";
    import { push_shader } from "/js_reader.js?url";
    import { onMount } from "svelte";
    import CodeMirror from "svelte-codemirror-editor";
    import { oneDark } from "@codemirror/theme-one-dark";
    import { wgsl } from "@iizukak/codemirror-lang-wgsl";

    let compute_value = `#import bevy_render::globals::Globals

struct OCTree {
    @location(0) offset: u32,
    @location(1) mask: u32,
}

struct Voxel {
    @location(0) col: u32,
    @location(1) mat: u32,
}

@group(0) @binding(0) var<uniform> globals: Globals;
@group(0) @binding(1) var<storage, read_write> octrees: array<OCTree>;
@group(0) @binding(2) var<storage, read_write> voxels: array<Voxel>;

fn sphere(pos: vec3<f32>, r: f32) -> f32 {

    return length(pos) - r;
}

fn map(pos: vec3<f32>) -> f32 {

    return sphere(pos, 0.25);
}

// fn calc_pos_from_invoc_id(global_id: vec3<u32>) -> vec3<f32> {


// }

// 8 total octrees and workers, this is the final level where we should calculate the voxels.
// steps this compute shader needs to take is as follows:
// 1. check if there is anything within bounds
// 2. update parent octree with result of within bounds
// 3. calculate all the voxels within its bounds (8 total)
@compute @workgroup_size(2, 2, 2)
fn init(@builtin(global_invocation_id) global_id: vec3<u32>) {

    if (octrees[0].offset == 0u) {
        octrees[0].offset = 1u;
        octrees[0].mask = 2u;

        voxels[0].col = 1u;

        return;
    }

    octrees[0].offset += 1u;
    octrees[0].mask += 1u;
    voxels[0].col += 1u;
}

@compute @workgroup_size(1, 1, 1)
fn finalize(@builtin(global_invocation_id) global_id: vec3<u32>) {

    if (octrees[0].offset == 1u) {
        octrees[1].offset = 1u;
        octrees[1].mask = 2u;

        voxels[1].col = 1u;
    }
}
`;

    let value = `#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
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

    col = vec4(sin(globals.time), cos(globals.time), 0.0, 1.0);

    if (octrees[1].offset == 1 && voxels[1].col == 1) {
        col = vec4(0., 1., 1., 1.);
    }

    var out: Output;
    out.history = vec4(col);
    out.view_target = vec4(col);
    return out;
}`;

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

<CodeMirror
    class="text_edit"
    bind:value
    lang={wgsl()}
    theme={oneDark}
    on:change={handle_change}
/>

<style>
    .text_edit {
        position: absolute;
    }
</style>
