<script>
    import init from "/shade/bevy_shade.js?url";
    import { push_shader } from "/js_reader.js?url";
    import { onMount } from "svelte";
    import CodeMirror from "svelte-codemirror-editor";
    import { oneDark } from "@codemirror/theme-one-dark";
    import { wgsl } from "@iizukak/codemirror-lang-wgsl";

    let compute_value = 
`struct OCTree {
    @location(0) offset: u32,
    @location(1) mask: u32,
}

@group(0) @binding(0) var<storage, read_write> data: array<OCTree>;

@compute @workgroup_size(128)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    data[0].offset = 1u;
    data[0].mask = 2u;
}
`;

    let value = `// This shader computes the chromatic aberration effect
#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_render::globals::Globals

struct OCTree {
    @location(0) offset: u32,
    @location(1) mask: u32,
}

@group(0) @binding(0) var<uniform> globals: Globals;
@group(0) @binding(1) var<storage, read> data: array<OCTree>;

@group(0) @binding(2) var screen_texture: texture_2d<f32>;
@group(0) @binding(3) var prev_frame: texture_2d<f32>;

@group(0) @binding(4) var nearest_sampler: sampler;
@group(0) @binding(5) var linear_sampler: sampler;

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

    if (data[0].offset == 1) {
        col = vec4(0., 1., 1., 1.);
    }

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
