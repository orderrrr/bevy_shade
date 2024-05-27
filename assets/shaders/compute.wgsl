// com fute shader
#import bevy_render::globals::Globals
#import octree::OCTreeSettings
#import octree::OCTreeRuntime

struct OCTree {
    offset: u32,
    mask: u32,
}

struct Voxel {
    col: u32,
    mat: u32,
}

@group(0) @binding(0) var<uniform> globals: Globals;
@group(0) @binding(1) var<uniform> settings: OCTreeSettings;
@group(0) @binding(2) var<uniform> runtime: OCTreeRuntime;
@group(0) @binding(3) var<storage, read_write> octrees: array<OCTree>;
@group(0) @binding(4) var<storage, read_write> voxels: array<Voxel>;

// 8 total octrees and workers, this is the final level where we should calculate the voxels.
// steps this compute shader needs to take is as follows:
// 1. check if there is anything within bounds
// 2. update parent octree with result of within bounds
// 3. calculate all the voxels within its bounds (8 total)
@compute @workgroup_size(1,1,1)
fn init(@builtin(global_invocation_id) global_id: vec3<u32>) {

    let i = runtime.depth;
    octrees[count_octrees_below(i, settings.depth) + get_unique_index_for_dim(global_id, i)].mask = 1u;

    // let b = settings.scale; // 2.0
    // let point = calc_pos_from_invoc_id(global_id, i);
    // let d = map(point);
    // let index = count_octrees_below(i, settings.depth) + get_unique_index_for_dim(global_id, runtime.depth);

    // octrees[index].mask = 1u;

    // // if d <= (b / f32(1u << i)) {
    // if true {

    //     for (var j: u32 = 0; j < 2; j++) {
    //         for (var k: u32 = 0; k < 2; k++) {
    //             for (var l: u32 = 0; l < 2; l++) {

    //                 let rvpos = vec3<u32>(j, k, l);
    //                 let vpos = get_child_pos(global_id, rvpos);
    //                 let vip = calc_pos_from_invoc_id(vpos, i + 1);
    //                 let voxid = get_unique_index_for_dim(vpos, i + 1);
    //                 let vidx = get_unique_index_for_dim(rvpos, 2u);
    //                 let d2 = map(vip);

    //                 // if d2 < (b / f32(1u << i + 1)) * 0.5 {
    //                 if true {

    //                     voxels[voxid].col = 1u;
    //                     voxels[voxid].mat = 1u;

    //                     octrees[index].mask = insertBits(octrees[index].mask, 1u, vidx, 1u);

    //                     continue;
    //                 }

    //                 voxels[voxid].col = 0u;
    //                 voxels[voxid].mat = 0u;
    //             }
    //         }
    //     }
    // }
}

// 1 worker which will calculate the
@compute @workgroup_size(1, 1, 1)
fn finalize(@builtin(global_invocation_id) global_id: vec3<u32>) {

    let i = runtime.depth;
    octrees[count_octrees_below(i, settings.depth) + get_unique_index_for_dim(global_id, i)].mask = 1u;

    // let index = count_octrees_below(i, settings.depth) + get_unique_index_for_dim(global_id, i);

    // let b = settings.scale;

    // let point = calc_pos_from_invoc_id(global_id, i);
    // let d = map(point);

    // octrees[index].mask = 1u;


    // octrees[index].mask = 0u;

    // for (var j: u32 = 0; j < 2; j++) {
    //     for (var k: u32 = 0; k < 2; k++) {
    //         for (var l: u32 = 0; l < 2; l++) {

    //             let vid = vec3<u32>(j, k, l);
    //             let vpos = get_child_pos(global_id, vid);
    //             let cid = count_octrees_below(i + 1, settings.depth) + get_unique_index_for_dim(vpos, i + 1);
    //             let child_octree = octrees[cid];

    //             if true {
    //                 let vidx = get_unique_index_for_dim(vid, 2u);

    //                 octrees[index].mask = insertBits(octrees[index].mask, 1u, vidx, 1u);
    //             }
    //         }
    //     }
    // }
}

fn sphere(pos: vec3<f32>, r: f32) -> f32 {

    return length(pos) - r;
}

fn map(pos: vec3<f32>) -> f32 {

    return sphere(pos, 2.0);
}


















































fn calc_pos_from_invoc_id(indices: vec3<u32>, i: u32) -> vec3<f32> {
    let scale = f32(settings.scale);
    let d = 1u << i;
    let size: f32 = scale / f32(d);

    let center = (vec3<f32>(indices) + 0.5) * size - (scale / 2.0);

    return center;
}

fn get_child_pos(parent_pos: vec3<u32  >, child_rel_pos: vec3<u32>) -> vec3<u32> {
    return parent_pos * 2 + child_rel_pos;
}

fn get_unique_index_for_dim(g: vec3<u32>, i: u32) -> u32 {
    let dim = 1u << i;
    return g.x + g.y * dim + g.z * dim * dim;
}

fn get_child_index(parent_pos: vec3<u32 >, child_rel_pos: vec3<u32>, parent_depth: u32) -> u32 {
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
