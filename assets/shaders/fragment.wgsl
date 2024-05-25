#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_render::globals::Globals
#import octree::OCTreeSettings

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

    let an = globals.time * 0.5;

    let resolution = vec2<f32>(textureDimensions(screen_texture));
    // custom uv, not quite the same as in.uv.
    var uv: vec2<f32> = (2. * in.position.xy - resolution.xy) / resolution.y;
    // invert uv.
    uv = uv * vec2(1.0, -1.0);

    // rotation around 0.,,
    let ro = vec3(5. * sin(an), 0.0, 5. * cos(an));

    // todo convert this from linear algebra rotation to geometric algebra.
    let ta = vec3(0.);
    let ww = normalize(ta - ro);
    let uu = normalize(cross(ww, vec3(0.0, 1.0, 0.0)));
    let vv = normalize(cross(uu, ww));

    // ray direction.
    let rd = normalize(uv.x * uu + uv.y * vv + 2.0 * ww);

    col = vec3(.1, .2, .8);
    col = mix(col, vec3(0.7, 0.75, 0.8), exp(-10. * rd.y));

    let t: f32 = cast_ray(ro, rd);

    // if its -1 leave blank and use sky color.
    if t > 0. {

        let pos = ro + t * rd;
        let nor = calc_normal(pos);
        let mate = vec3(0.18);

        let sun_dir = normalize(vec3(.8, .4, .4));
        let sun_dif = clamp(dot(nor, sun_dir), 0., 1.);
        let sun_sha = step(cast_ray(pos + nor * 0.001, sun_dir), 0.);

        let sky_dif = clamp(.5 + .5 * dot(nor, vec3(0.1, 1.0, 0.)), 0., 1.);
        let bou_dif = clamp(.5 + .5 * dot(nor, vec3(0., -1., 0.)), .0, 1.);

        col = mate * vec3(7., 4.5, 3.) * sun_dif * sun_sha;
        col += mate * vec3(0.5, 0.8, 0.9) * sky_dif;
        col += mate * vec3(1., 0.4, 0.3) * bou_dif;
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

    // for loop through octree 8next.
    for (var i: u32 = 0; i < 100; i++) {

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

fn map(pos: vec3<f32>) -> f32 {

    let d1 = length(pos) - 1;
    let d2 = pos.y - (-1.4);

    let oc = distance_to_octree(pos);


    return min(oc, d2);
}

fn distance_to_octree(p: vec3<f32>) -> f32 {

    let d = 1u << 0u;

    // var grid_pos = round(p, settings.scale / f32(d));
    var grid_pos = vec3<u32>(0);

    let p1 = calc_pos_from_invoc_id(grid_pos, 0u);
    let gid = get_unique_index_for_dim(grid_pos, 0u);

    let octree = octrees[gid];

    var h = 1000.;

    // check if we have data
    // if octree.mask > 0u {
    if true {

        let d1 = cube(p - p1, vec3((settings.scale / f32(d)) / 2.0));

        if d1 < (settings.scale / f32(d) + 0.4) {
            h = cube_frame(p - p1, vec3((settings.scale / (f32(d) * 2.0))), 0.01);
            if (h <= 0.001) {
                return h;
            }

            for (var i: u32 = 1; i > settings.depth; i++) {

                let depth = 1u << i;
                var prev_idx = count_octrees_below(i, settings.depth);

                var any_match = false;

                for (var l: u32 = 0; l < depth; l++) {
                    for (var m: u32 = 0; m < depth; m++) {
                        for (var n: u32 = 0; n < depth; n++) {

                            let cpos = vec3<u32>(l, m, n);
                            // let index = count_octrees_below(depth - 1, settings.depth) + get_unique_index_for_dim(cpos, depth);
                            let ip = calc_pos_from_invoc_id(cpos, i);
                            let id = get_unique_index_for_dim(cpos, i);
                            let coctree = octrees[prev_idx + id];

                            // h = min(cube_frame(p - ip, vec3(settings.scale / f32((1u << i)) / 2.0), 0.004), h);

                            if coctree.mask > 0u {
                                h = min(cube_frame(p - ip, vec3(settings.scale / f32((1u << i)) / 2.0), 0.004), h);

                                if (h <= 0.001) {
                                    return h;
                                }

                                any_match = true;

                                if i == settings.depth {

                                    for (var q: u32 = 0; q < 2; q++) {
                                        for (var r: u32 = 0; r < 2; r++) {
                                            for (var s: u32 = 0; s < 2; s++) {

                                                let vpos = vec3<u32>(q, r, s);
                                                let vox_pos = get_child_pos(cpos, vpos);
                                                let vip = calc_pos_from_invoc_id(vox_pos, i + 1);
                                                let voxid = get_unique_index_for_dim(vox_pos, i + 1);

                                                let voxel = voxels[voxid];

                                                if voxel.col > 0u {
                                                    h = min(cube(p - vip, vec3((settings.scale / (f32(i + 1) * 16.)))), h);
                                                }
                                            }
                                        }
                                    }

                                    if (h <= 0.001) {
                                        return h;
                                    }
                                }
                            }
                        }
                    }
                }

                if !any_match {

                    break;
                }
            }
        }
    }



    return h;
}

// Helper function to round a single float to uint.
fn round(v: vec3<f32>, s: f32) -> vec3<u32> {
    // Round the value and convert to u32.
    return vec3<u32>(floor(v - (s * 2.0)));
}

fn calc_normal(pos: vec3<f32>) -> vec3<f32> {

    let e = vec2(0.0001, 0.0);

    return normalize(vec3(
        map(pos + e.xyy) - map(pos - e.xyy),
        map(pos + e.yxy) - map(pos - e.yxy),
        map(pos + e.yyx) - map(pos - e.yyx),
    ));
}

//      pos           dims.
fn cube(p: vec3<f32>, d: vec3<f32>) -> f32 {
    let q = abs(p) - d;
    return length(max(q, vec3(0.0))) + min(max(q.x, max(q.y, q.z)), 0.);
}

fn cube_frame(p: vec3<f32>, b: vec3<f32>, e: f32) -> f32 {
    var p1 = abs(p) - b;
    let q = abs(p1 + vec3<f32>(e)) - vec3<f32>(e);
    return min(min(
        length(max(vec3<f32>(p1.x, q.y, q.z), vec3<f32>(0.0))) + min(max(p1.x, max(q.y, q.z)), 0.0),
        length(max(vec3<f32>(q.x, p1.y, q.z), vec3<f32>(0.0))) + min(max(q.x, max(p1.y, q.z)), 0.0)
    ),
        length(max(vec3<f32>(q.x, q.y, p1.z), vec3<f32>(0.0))) + min(max(q.x, max(q.y, p1.z)), 0.0));
}















































fn calc_pos_from_invoc_id(indices: vec3<u32>, i: u32) -> vec3<f32> {
    let scale = f32(settings.scale);
    let d = 1u << i;
    let size: f32 = scale / f32(d);

    let center = (vec3<f32>(indices) + 0.5) * size - (scale / 2.0);

    return center;
}

fn get_child_pos(parent_pos: vec3<u32>, child_rel_pos: vec3<u32>) -> vec3<u32> {
    return parent_pos * 2 + child_rel_pos;
}

fn get_unique_index_for_dim(g: vec3<u32>, i: u32) -> u32 {
    let dim = 1u << i;
    return g.x + g.y * dim + g.z * dim * dim;
}

fn get_child_index(parent_pos: vec3<u32>, child_rel_pos: vec3<u32>, parent_depth: u32) -> u32 {
    let pos = get_child_pos(parent_pos, child_rel_pos);
    return get_unique_index_for_dim(pos, parent_depth + 1);
}

fn get_position_from_unique_index(index: u32, i: u32) -> vec3<u32> {
    let d = 1u << i;
    let z = index / (d * d);
    let remaining = index % (d * d);
    let y = remaining / d;
    let x = remaining % d;
    return vec3<u32>(x, y, z);
}

fn count_octrees_below(cd: u32, i: u32) -> u32 {
    return u32(pow(8.0, f32(i + 1u)) / 7.0 - pow(8.0, f32(cd + 1u)) / 7.0);
}
