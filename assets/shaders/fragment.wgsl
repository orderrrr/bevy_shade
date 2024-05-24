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
    let an = globals.time * 0.5;

    let resolution = vec2<f32>(textureDimensions(screen_texture));
    // custom uv, not quite the same as in.uv.
    var uv: vec2<f32>  = (2. * in.position.xy - resolution.xy) / resolution.y;
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
    let rd = normalize(uv.x*uu + uv.y*vv + 2.0*ww);

    col = vec3(.1, .2, .8);
    col = mix(col, vec3(0.7, 0.75, 0.8), exp(-10. * rd.y));

    let t: f32 = cast_ray(ro, rd);

    // if its -1 leave blank and use sky color.
    if (t > 0.) {

        let pos = ro + t * rd;
        let nor = calc_normal(pos);
        let mate = vec3(0.18);

        let sun_dir = normalize(vec3(.8, .4, 0.));
        let sun_dif = clamp(dot(nor, sun_dir), 0., 1.);
        let sun_sha = step(cast_ray(pos+nor * 0.001, sun_dir), 0.);

        let sky_dif = clamp(.5 + .5 * dot(nor, vec3(0.1, 1.0, 0.)), 0., 1.);
        let bou_dif = clamp(.5 + .5 * dot(nor, vec3(0., -1., 0.)), .0, 1.);

        col  = mate*vec3(7., 4.5, 3.) * sun_dif * sun_sha;
        col += mate*vec3(0.5, 0.8, 0.9) * sky_dif;
        col += mate*vec3(1., 0.4, 0.3) * bou_dif;
    }

    // gamma correct
    col = pow(col, vec3(0.4545));

    // col = mix(textureSample(prev_frame, linear_sampler, in.uv).xyz, col, 0.001);

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

fn map(pos: vec3<f32>) -> f32{

    let d1 = length(pos) - 1;
    let d2 = pos.y - (-1.4);

    let oc = distance_to_octree(pos, 0u);


    return min(oc, d2);
}

fn distance_to_octree(p: vec3<f32>, i: u32) -> f32 {
    let d = u32(pow(2., f32(i)));

    // var grid_pos = round(p, settings.scale / f32(d));
    var grid_pos = vec3<u32>(0);

    let p1 = calc_pos_from_invoc_id(grid_pos, i);
    let gid = get_unique_index_for_dim(grid_pos, i);

    let octree = octrees[gid];

    var h = 1000.;

    // check if we have data
    if octree.mask > 0u {

        h = cube(p - p1, vec3((settings.scale / f32(d)) / 2.0));

        if h < (settings.scale / f32(d) + 0.4) {
            h = cube_frame(p - p1, vec3((settings.scale / (f32(d) * 2.0))), 0.01);

            for (var i: u32 = 1; i < settings.depth + 1; i++) {

                let depth = u32(pow(2., f32(i)));

                for (var l: u32 = 0; l < depth; l++) {
                    for (var m: u32 = 0; m < depth; m++) {
                        for (var n: u32 = 0; n < depth; n++) {

                            let cpos = vec3<u32>(l, m, n);
                            // let index = count_octrees_below(depth - 1, settings.depth) + get_unique_index_for_dim(cpos, depth);
                            let ip = calc_pos_from_invoc_id(cpos, i);
                            let id = get_unique_index_for_dim(cpos, i);
                            let coctree = octrees[id];

                            if coctree.mask > 0u {
                                h = min(cube_frame(p - ip, vec3((settings.scale / (f32(i) * 4.0))), 0.004), h);

                                for (var q: u32 = 0; q < 2; q++) {
                                    for (var r: u32 = 0; r < 2; r++) {
                                        for (var s: u32 = 0; s < 2; s++) {

                                            let vpos = vec3<u32>(q, r, s);
                                            // voxel
                                            let vip = calc_vpos_from_vid_and_parent(ip, vpos, i);
                                            let offset = (grid_pos * 2);
                                            let voxid = get_child_index(cpos, vpos, 1u);


                                            // |    0|    1|    2|    3|
                                            // | 0| 1| 0| 1| 0| 1| 0| 1|
                                            // | 0| 1| 2| 3| 4| 5| 6| 7|
                                            

                                            // this
                                            // 1,0,0
                                            // 0,0,0

                                            // is the same as this
                                            // 0,0,0
                                            // 1,0,0

                                            // (0 + 0) + ((0 + 0) * 8) + ((0 + 1) * 8 * 8)


                                            let voxel = voxels[voxid];

                                            if voxel.col > 0u {
                                                h = min(cube(p - vip, vec3((settings.scale / (f32(i + 1) * 16.)))), h);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // let g2 = round(p, settings.scale / f32(dep));
                // let i2 = count_octrees_below(dep - 1, settings.depth) + get_unique_index_for_dim(g2, dep);

                // let t2 = octrees[i2];

                // var h2 = cube(p, vec3(settings.scale / f32(dep)) + 0.4);

                // if h2 < (settings.scale / f32(dep) + 0.4) {
                //     h = min(cube_frame(p, vec3(settings.scale / f32(dep)), 0.01), h);
                // }
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
        map(pos+e.xyy) - map(pos-e.xyy),
        map(pos+e.yxy) - map(pos-e.yxy),
        map(pos+e.yyx) - map(pos-e.yyx),
    ));
}

//      pos           dims.
fn cube(p: vec3<f32>, d: vec3<f32>) -> f32 {
    let q = abs(p) - d;
    return length(max(q,vec3(0.0))) + min(max(q.x,max(q.y,q.z)),0.);
}

fn cube_frame(p: vec3<f32>, b: vec3<f32>, e: f32) -> f32 {
    var p1 = abs(p) - b;
    let q = abs(p1 + vec3<f32>(e)) - vec3<f32>(e);
    return min(min(
        length(max(vec3<f32>(p1.x, q.y, q.z), vec3<f32>(0.0))) + min(max(p1.x, max(q.y, q.z)), 0.0),
        length(max(vec3<f32>(q.x, p1.y, q.z), vec3<f32>(0.0))) + min(max(q.x, max(p1.y, q.z)), 0.0)),
        length(max(vec3<f32>(q.x, q.y, p1.z), vec3<f32>(0.0))) + min(max(q.x, max(q.y, p1.z)), 0.0)
    );
}

















































fn calc_pos_from_invoc_id(block_indices: vec3<u32>, i: u32) -> vec3<f32> {
    let scale = settings.scale / pow(2., f32(i));
    var offset = scale * 0.5;

    if i == 0u {
        offset = 0.0;
    }

    return vec3<f32>(block_indices) * scale - offset;
}

fn calc_vpos_from_vid_and_parent(ppos: vec3<f32>, child_offset: vec3<u32>, parent_depth: u32) -> vec3<f32> {
    let child_pos = calc_pos_from_invoc_id(child_offset, 1u);
    return (child_pos * 0.5) + ppos;
}

fn get_unique_index_for_dim(g: vec3<u32>, i: u32) -> u32 {
    let dim = u32(pow(2., f32(i)));
    return g.x + g.y * dim + g.z * dim * dim;
}

fn get_child_index(parent_pos: vec3<u32>, child_rel_pos: vec3<u32>, parent_depth: u32) -> u32 {
    let parent_size = vec3<u32>(1u << parent_depth);
    let child_grid_size = parent_size * 2u;

    let parent_index = parent_pos.x * parent_size.y * parent_size.z +
                       parent_pos.y * parent_size.z +
                       parent_pos.z;

    let child_offset = child_rel_pos.x * (1u << parent_depth) +
                       child_rel_pos.y * (1u << (parent_depth + 1u)) +
                       child_rel_pos.z * (1u << (parent_depth + 2u));

    let child_index = parent_index * 8u + child_offset;

    return child_index;
}

fn get_position_from_unique_index(index: u32, i: u32) -> vec3<u32> {
    let d = u32(pow(2., f32(i)));
    let z = index / (d * d);
    let remaining = index % (d * d);
    let y = remaining / d;
    let x = remaining % d;
    return vec3<u32>(x, y, z);
}

fn count_octrees_below(cd: u32, i: u32) -> u32 {
    return u32(pow(8.0, f32(i + 1u)) / 7.0 - pow(8.0, f32(cd + 1u)) / 7.0);
}
