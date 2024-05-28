#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_render::globals::Globals

#import octree::{
    OCTreeSettings,
    OCTreeRuntime,
    Voxel,
    OCTree,
    get_enclosed_octree,
    calc_pos_from_invoc_id,
    get_child_pos,
    get_unique_index_for_dim,
    get_child_index,
    get_position_from_unique_index,
    count_octrees_below
}

const AA: i32 = 2;
const DEBUG: bool = false;

@group(0) @binding(0) var<uniform> globals: Globals;
@group(0) @binding(1) var<storage, read> octrees: array<OCTree>;
@group(0) @binding(2) var<storage, read> voxels: array<Voxel>;
@group(0) @binding(3) var<uniform> settings: OCTreeSettings;

@group(0) @binding(4) var screen_texture: texture_2d<f32>;
@group(0) @binding(5) var prev_frame: texture_2d<f32>;

@group(0) @binding(6) var nearest_sampler: sampler;
@group(0) @binding(7) var linear_sampler: sampler;








// TODO octree parser can be greatly improved by:

// get the octree at the current pos,
// if it is not filled, get the next possible position and move p based on its rayDirection
// if it is filled but the voxel was not hit, either get the next voxel or get the next octree up.
// based on wether or not its the edge of the octree or not,















struct Output {
  @location(0) view_target: vec4<f32>,
  @location(1) history: vec4<f32>,
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> Output {

    var total = vec3(0.0);

    // for AA
    for (var m: i32 = 0; m < AA; m++) {
        for (var n: i32 = 0; n < AA; n++) {
            var col = vec3(sin(globals.time), cos(globals.time + 5.0), cos(sin(globals.time + 1.0)));

            let an = globals.time * 0.5;

            let resolution = vec2<f32>(textureDimensions(screen_texture));
            let o = vec2(f32(m), f32(n)) / f32(AA) - 0.5;

            // custom uv, not quite the same as in.uv.
            var uv: vec2<f32> = (2. * (in.position.xy + o) - resolution.xy) / resolution.y;
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
            let rd: vec3<f32> = normalize(uv.x * uu + uv.y * vv + 2.0 * ww);

            col = vec3(.1, .2, .8);
            col = mix(col, vec3(0.7, 0.75, 0.8), exp(-10. * rd.y));

            render(ro, rd, &col);

            // gamma correct
            col = pow(col, vec3(0.4545));

            total = total + col;
        }
    }

    total = total / f32(AA * AA);

    // s-surve
    total = clamp(total, vec3<f32>(0.0), vec3<f32>(1.0));
    total = total * total * (3.0 - 2.0 * total);

    var out: Output;
    out.history = vec4(total, 1.);
    out.view_target = vec4(total, 1.);
    return out;
}

fn render(ro: vec3<f32>, rd: vec3<f32>, col: ptr<function, vec3<f32>>) {
    let t: f32 = cast_ray(ro, rd);

    if (t == -1) {
        *col = vec3<f32>(0.0);  
        return;
    }

    if (t <= 0.01) {
        *col = vec3<f32>(1.0);
        return;
    }

    // var pos = vec3<f32>(0.0);

    // // if its -1 leave blank and use sky color.
    // if t > 0. {

    //     let pos = ro + t * rd;
    //     let nor = calc_normal(pos);
    //     let mate = vec3(0.18);

    //     let sun_dir = normalize(vec3(.8, .4, .4));
    //     let sun_dif = clamp(dot(nor, sun_dir), 0., 1.);
    //     let sun_sha = step(cast_ray(pos + nor * 0.001, sun_dir), 0.);

    //     let sky_dif = clamp(.5 + .5 * dot(nor, vec3(0.1, 1.0, 0.)), 0., 1.);
    //     let bou_dif = clamp(.5 + .5 * dot(nor, vec3(0., -1., 0.)), .0, 1.);

    //     *col = mate * vec3(7., 4.5, 3.) * sun_dif * sun_sha;
    //     *col += mate * vec3(0.5, 0.8, 0.9) * sky_dif;
    //     *col += mate * vec3(1., 0.4, 0.3) * bou_dif;
    // }
}

fn cast_ray(ro: vec3<f32>, rd: vec3<f32>) -> f32 {

    var t = 0.;
    var depth = 0u;

    // for loop through octree 8next.
    for (var i: u32 = 0; i < 100; i++) {

        let pos = ro + t * rd;

        let h = distance_to_octree(pos, rd, &depth);

        if h < 0.001 && depth == settings.depth {
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

fn map(pos: vec3<f32>, rd: vec3<f32>) -> f32 {

    let d1 = length(pos) - 1;
    let d2 = pos.y - (-1.4);

    var depth = settings.depth;

    // this needs to be more friendly to the raycast.
    let oc = distance_to_octree(pos, rd, &depth);

    // return min(oc, d2);
    return oc;
}

fn get_next_octree_pos(rp: vec3<f32> , rd: vec3<f32>, i: u32, scale: f32) -> vec3<i32> {

    let d = 1u << i;

    // normalize the rd and multiply by the size of the current dim voxel, putting us somewhere inside the next grid
    let np = rp + (normalize(rd) * (scale / f32(i)));

    // get the next octree in that position.
    return get_enclosed_octree(np, d, settings.scale); // grid position
}

fn distance_to_octree(rp: vec3<f32>, rd: vec3<f32>, depth: ptr<function, u32>) -> f32 {

    if *depth == settings.depth {
        // we are at the lowest, get the closest of all voxels.
        let gp = vec3<u32>(get_nth_octree_pos(rp, rd, *depth, settings.scale));

        var dist = settings.scale;

        for (var j: u32 = 0; j < 2; j++) {
            for (var k: u32 = 0; k < 2; k++) {
                for (var l: u32 = 0; l < 2; l++) {

                    let v = vec3<u32>(j, k, l);
                    let voxel = get_child_pos(gp, v);

                    let id = get_unique_index_for_dim(voxel, *depth);
                    let oc = voxels[id];

                    if oc.col < 1u {

                        continue;
                    }

                    let po = calc_pos_from_invoc_id(voxel, *depth, settings.scale);

                    dist = min(get_dist(rp - po, *depth), dist);
                }
            }
        }

        return dist;
    }

    var dist = get_dist_for_dim(rp, rd, *depth);

    if dist <= 0.001 {
        // *depth = settings.depth;
        *depth = *depth + 1;
        dist = get_dist_for_dim(rp, rd, *depth);
    }

    if *depth == settings.depth {
        // we are at the lowest, get the closest of all voxels.
        let gp = vec3<u32>(get_nth_octree_pos(rp, rd, *depth, settings.scale));

        var dist = settings.scale;

        for (var j: u32 = 0; j < 2; j++) {
            for (var k: u32 = 0; k < 2; k++) {
                for (var l: u32 = 0; l < 2; l++) {

                    let v = vec3<u32>(j, k, l);
                    let voxel = get_child_pos(gp, v);

                    let id = get_unique_index_for_dim(voxel, *depth);
                    let oc = voxels[id];

                    if oc.col < 1u {

                        continue;
                    }

                    let po = calc_pos_from_invoc_id(voxel, *depth, settings.scale);

                    dist = min(get_dist(rp - po, *depth), dist);
                }
            }
        }

        return dist;
    }

    return dist;

    // var i = dim;
    // let d = 1u << i;

    // var h = settings.scale / 2.0;

    // var gpi = get_enclosed_octree(rp, 1u, settings.scale); // grid position
    // var gp = vec3<u32>(0u);

    // let pos = calc_pos_from_invoc_id(gp, i, settings.scale);
    // let dist = get_dist(rp - pos, i);
    
    // if (dist > 0.001) {
    //     return dist;
    // }

    // let index = count_octrees_below(i, settings.depth) + get_unique_index_for_dim(gp, i);
    // let octree = octrees[index];

    // // first octree , exit if we are too far from the octree or nothing is there, skip.
    // if octree.mask > 0u && dist < (settings.scale / f32(1u << i)) + 0.1 {

    //     // draw debug frame, exit if we hit it.
    //     if DEBUG {
    //         h = cube_frame(rp - pos, vec3((settings.scale / f32(d)) / 2.0), 0.002);
    //     }

    //     if h < 0.001 {
    //         return h;
    //     }

    //     // final depth should probably just check the voxels around it as well.
    //     while i < settings.depth + 1 {

    //         i = i + 1;

    //         if i == settings.depth + 1u {

    //             // get closest voxel octree that is active.
    //             for (var j: u32 = 0; j < 2; j++) {
    //                 for (var k: u32 = 0; k < 2; k++) {
    //                     for (var l: u32 = 0; l < 2; l++) {

    //                         let v = vec3<u32>(j, k, l);
    //                         let closest = get_child_pos(gp, v);

    //                         let id = get_unique_index_for_dim(closest, i);
    //                         let oc = voxels[id];

    //                         if oc.col < 1u {

    //                             continue;
    //                         }

    //                         let po = calc_pos_from_invoc_id(closest, i, settings.scale);

    //                         h = min(get_dist(rp - po, i), h);

    //                         if DEBUG {
    //                             let po = calc_pos_from_invoc_id(closest, i, settings.scale);
    //                             h = min(cube_frame(rp - po, vec3((settings.scale) / f32(1u << i)) / 2.0, 0.002), h);
    //                         }
    //                     }
    //                 }
    //             }


    //             return h;
    //         }

    //         var dist = 100.;
    //         var co = vec3<u32>(0);

    //         var octrees_below = count_octrees_below(i, settings.depth);

    //         // get closest voxel octree that is active.
    //         for (var j: u32 = 0; j < 2; j++) {
    //             for (var k: u32 = 0; k < 2; k++) {
    //                 for (var l: u32 = 0; l < 2; l++) {

    //                     let v = vec3<u32>(j, k, l);
    //                     let closest = get_child_pos(gp, v);

    //                     let id = octrees_below + get_unique_index_for_dim(closest, i);
    //                     let oc = octrees[id];

    //                     if oc.mask < 1u {

    //                         continue;
    //                     }

    //                     let po = calc_pos_from_invoc_id(closest, i, settings.scale);

    //                     let cdist = get_dist(rp - po, i);

    //                     if cdist < dist {
    //                         dist = cdist;
    //                         co = closest;
    //                     }
    //                 }
    //             }
    //         }

    //         if dist == 100. {
    //             return h;
    //         }

    //         gp = co;

    //         let po = calc_pos_from_invoc_id(co, i, settings.scale);

    //         if DEBUG {
    //             h = min(cube_frame(rp - po, vec3((settings.scale) / f32(1u << i)) / 2.0, 0.002), h);
    //         }

    //         if h < 0.001 || dist > (settings.scale / f32(1u << i - 1)) + 0.1 {
    //             return h;
    //         }
    //     }
    // }

    // return h;
}

fn get_dist(pos: vec3<f32>, i: u32) -> f32 {

    return cube(pos, vec3((settings.scale / f32(1u << i)) / 2.0));
}

// Helper function to round a single float to uint.
fn round(v: vec3<f32>, s: f32) -> vec3<u32> {
    // Round the value and convert to u32.
    return vec3<u32>(floor(v - (s * 2.0)));
}

fn calc_normal(pos: vec3<f32>) -> vec3<f32> {

    let e = vec2(0.0001, 0.0);

    return normalize(vec3(
        map(pos + e.xyy, vec3<f32>(0.0)) - map(pos - e.xyy, vec3<f32>(0.0)),
        map(pos + e.yxy, vec3<f32>(0.0)) - map(pos - e.yxy, vec3<f32>(0.0)),
        map(pos + e.yyx, vec3<f32>(0.0)) - map(pos - e.yyx, vec3<f32>(0.0)),
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

































fn get_octree_pos(rp: vec3<f32>, rd: vec3<f32>, i: u32, scale: f32) -> vec3<i32> {

    // get the next octree in that position.
    return get_enclosed_octree(rp, 1u << i, settings.scale); // grid position
}

fn get_nth_octree_pos(rp: vec3<f32>, rd: vec3<f32>, d: u32, scale: f32) -> vec3<i32> {

    // normalize the rd and multiply by the size of the current dim voxel, putting us somewhere inside the next grid
    let np = rp + (normalize(rd) * (scale / f32(d)));

    // get the next octree in that position.
    return get_enclosed_octree(np, 1u << d, settings.scale); // grid position
}

fn get_distance_to_next_octree(gp: vec3<u32>, rp: vec3<f32>, rd: vec3<f32>, i: u32, scale: f32) -> f32 {

    let index = count_octrees_below(i, settings.depth) + get_unique_index_for_dim(gp, i);
    let octree = octrees[index];

    if octree.mask < 1u {
        return settings.scale / f32(1u << i);
    }

    let pos = calc_pos_from_invoc_id(gp, i, settings.scale);

    // get the distance from the old position to the next octree
    return get_dist(rp - pos, i);
}

// currently we only have one octree, i would like to have multiple but for now assuming only one.
fn valid_octree_pos(gp: vec3<i32>, i: u32) -> bool {

    // make sure it is within the octree bounds.
    return
        gp.x > -1 && 
        gp.y > -1 && gp.z > -1 && gp.y < i32((1u << i)) && gp.x < i32((1u << i)) && gp.z < i32((1u << i));
}


fn get_next_dist_for_dim(rp: vec3<f32>, rd: vec3<f32>, i: u32) -> f32 {
    var ngp = get_nth_octree_pos(rp, rd, i, settings.scale);
    var ngpu = vec3<u32>(ngp);

    return get_distance_to_next_octree(ngpu, rp, rd, i, settings.scale);
}

fn get_dist_for_dim(rp: vec3<f32>, rd: vec3<f32>, i: u32) -> f32 {

    var gp = get_octree_pos(rp, rd, i, settings.scale);
    var gpu = vec3<u32>(gp);

    var ngp = get_nth_octree_pos(rp, rd, i, settings.scale);
    var ngpu = vec3<u32>(ngp);

    var dist = settings.scale / f32(1u << i);

    if valid_octree_pos(gp, i) {
        dist = get_distance_to_next_octree(gpu, rp, rd, i, settings.scale);
    }
        
    if dist > 0.001 && valid_octree_pos(ngp, i) {
        dist = get_distance_to_next_octree(ngpu, rp, rd, i, settings.scale);
    }

    return dist;
}

// fn dive(dist: f32, rp: vec3<f32>, rd: vec3<f32>, dim: u32) -> f32 {

//     var i = dim + 1u;
//     var d = (settings.scale / f32(1u << i + 1)); // max possible move if no match.

//     let check_current = get_next_dist_for_dim(rp, rd, i, 0u);





//     // var i = dim + 1u;
//     // var d = dist;

//     // d = get_next_dist_for_dim(rp, rd, i, 0u);

//     // while d <= 0.001 {

//     //     if i == settings.depth {
//     //         return d;
//     //     }

//     //     i = i + 1;
//     //     d = get_next_dist_for_dim(rp, rd, i, 0u);

//     //     if (d >= 0.001) {
//     //         d = get_next_dist_for_dim(rp, rd, i, 1u);
//     //     }
//     // }

//     // return d;
// }
