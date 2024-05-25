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

struct OCTreeSettings {
    depth: u32,
    scale: f32,
}

@group(0) @binding(0) var<uniform> globals: Globals;
@group(0) @binding(1) var<storage, read> octrees: array<OCTree>;
@group(0) @binding(2) var<storage, read> voxels: array<Voxel>;
@group(0) @binding(3) var<uniform> settings: OCTreeSettings;

@group(0) @binding(4) var screen_texture: texture_2d<f32>;
@group(0) @binding(5) var prev_frame: texture_2d<f32>;

@group(0) @binding(6) var nearest_sampler: sampler;
@group(0) @binding(7) var linear_sampler: sampler;

struct Output {
  @location(0) view_target: vec4<f32>,
  @location(1) history: vec4<f32>,
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> Output {
    var col = vec3(sin(globals.time), cos(globals.time + 5.0), cos(sin(globals.time + 1.0)));
    let an = globals.time;

    let resolution = vec2<f32>(textureDimensions(screen_texture));
    // custom uv, not quite the same as in.uv.
    let uv: vec2<f32>  = (2. * in.position.xy - resolution) / resolution.y;

    // rotation around 0.,,
    let ro = vec3(2.0 * sin(an), 0.0, 2.0 * cos(an));

    // todo convert this from linear algebra rotation to geometric algebra.
    let ta = vec3(0.);
    let ww = normalize(ta - ro);
    let uu = normalize(cross(ww, vec3(0.0, 1.0, 0.0)));
    let vv = normalize(cross(uu, ww));

    // ray direction.
    let rd = normalize(uv.x*uu + uv.y*vv + 1.0*ww);

    col = vec3(.1, .2, .8);
    col = mix(col, vec3(0.7, 0.75, 0.8), exp(10. * rd.y));

    let t: f32 = cast_ray(ro, rd);

    // if its -1 leave blank and use sky color.
    if (t > 0.) {

        let pos = ro + t * rd;
        let nor = calc_normal(pos);
        let mate = vec3(0.18);

        col += nor;

    }




    // gamma correct
    col = pow(col, vec3(0.4545));

    var out: Output;
    out.history = vec4(col, 1.);
    out.view_target = vec4(col, 1.);
    return out;
}

fn cast_ray(ro: vec3<f32>, rd: vec3<f32>) -> f32 {

    var t = 0.;

    // for loop through octree next.
    for (var i: u32 = 0; i < 2; i++) {

        let pos = ro + t * rd;
        let h = map(pos);

        if h < 0.001 {
            break;
        }

        t += h;

        if t > 20. {
            break;
        }
    }

    if t > 20. {
        t = -1.;
    }

    return t;
}

fn map(pos: vec3<f32>) -> f32{

    let d = length(pos) - 1;

    return d;
}

fn calc_normal(pos: vec3<f32>) -> vec3<f32> {

    let e = vec2(0.0001, 0.0);

    return normalize(vec3(
        map(pos+e.xyy) - map(pos-e.xyy),
        map(pos+e.xyx) - map(pos-e.xyx),
        map(pos+e.yyx) - map(pos-e.yyx),
    ));
}
