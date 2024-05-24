// compute shader
#import bevy_render::globals::Globals

struct OCTreeSettings {
    depth: u32,
    scale: f32,
}

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
@group(0) @binding(2) var<storage, read_write> octrees: array<OCTree>;
@group(0) @binding(3) var<storage, read_write> voxels: array<Voxel>;

// 8 total octrees and workers, this is the final level where we should calculate the voxels.
// steps this compute shader needs to take is as follows:
// 1. check if there is anything within bounds
// 2. update parent octree with result of within bounds
// 3. calculate all the voxels within its bounds (8 total)
@compute @workgroup_size(2, 2, 2)
fn init(@builtin(local_invocation_id) g: vec3<u32>, @builtin(num_workgroups) n: vec3<u32>) {
    init_with_dims(g, vec3(2u));
}

// 1 worker which will calculate the
@compute @workgroup_size(1, 1, 1)
fn finalize(@builtin(local_invocation_id) global_id: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>) {
    finalize_with_dims(global_id, vec3(1u));
}

fn sphere(pos: vec3<f32>, r: f32) -> f32 {

    return length(pos) - r;
}

fn map(pos: vec3<f32>) -> f32 {

    return sphere(pos + vec3<f32>(0.25), 0.7);
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

fn init_with_dims(global_id: vec3<u32>, num_workgroups: vec3<u32>) {

    let gidx_test = vec3<u32>(0u, 0u, 0u);
    let vidx_test = vec3<u32>(1u, 1u, 1u);

    let b = settings.scale; // 1.0

    let i = num_workgroups.x / 2;

    // depth is 2.0 - 1 = 1.0, gid 0,0,0
    let point = calc_pos_from_invoc_id(global_id, i);

    let d = map(point);

    let index = get_unique_index_for_dim(global_id, i);

    octrees[index].mask = 0u;

    if !(global_id.x == gidx_test.x && global_id.y == gidx_test.y && global_id.z == gidx_test.z) {
        octrees[index].mask = insertBits(octrees[index].mask, 1u, 0u, 1u);
        return;
    }

    if d <= b * 0.5 {
        // we hit an object.
        // find the correct index.

        // this is in init so we need to calc all voxels.
        for (var j: u32 = 0; j < 2; j++) {
            for (var k: u32 = 0; k < 2; k++) {
                for (var l: u32 = 0; l < 2; l++) {

                    let vid = vec3<u32>(j, k, l);
                    let pos = calc_vpos_from_vid_and_parent(point, vid, i);
                    let d2 = map(pos);

                    let vidi = get_child_index(global_id, vid, i);
                    let vidx = get_unique_index_for_dim(vid, 2u);

                    if global_id.x == gidx_test.x && global_id.y == gidx_test.y && global_id.z == gidx_test.z {

                        if vid.x == vidx_test.x && vid.y == vidx_test.y && vid.z == vidx_test.z {
                            voxels[vidi].col = 1u;
                            voxels[vidi].mat = 1u;
                            octrees[index].mask = insertBits(octrees[index].mask, 1u, vidx, 1u);
                            break;
                        }
                    }


                    // if (d2 < 0.1) {

                    //     voxels[vidx].col = 1u;
                    //     voxels[vidx].mat = 1u;

                    //     octrees[index].mask = insertBits(octrees[index].mask, 1u, vidx, 1u);

                    //     break;
                    // }

                    voxels[vidi].col = 0u;
                    voxels[vidi].mat = 0u;
                }
            }
        }
    }
}

fn finalize_with_dims(global_id: vec3<u32>, num_workgroups: vec3<u32>) {

    let index = count_octrees_below(num_workgroups.x - 1, settings.depth) + get_unique_index_for_dim(global_id, num_workgroups.x);

    let b = settings.scale;

    let point = calc_pos_from_invoc_id(global_id, num_workgroups.x - 1);
    let d = map(point);

    if d <= b {

        octrees[index].mask = 0u;

        for (var i: u32 = 0; i < 2; i++) {
            for (var j: u32 = 0; j < 2; j++) {
                for (var k: u32 = 0; k < 2; k++) {

                    let vid = vec3<u32>(i, j, k);
                    let ci = get_child_index(global_id, vid, num_workgroups.x);

                    if octrees[ci].mask > 1 {
                        let vidx = get_unique_index_for_dim(vid, 2u);

                        octrees[index].mask = insertBits(octrees[index].mask, 1u, vidx, 1u);
                    }
                }
            }
        }
    }
}
















// fn bit_insert(in: u32, gpos: vec3<u32>, offset: vec3<u32>, dim: u32) -> u32 {
//     // would come in as 2, so 2*2 gives 4
//     let dim = u32(pow(2., f32(dim)));
// 
//     // for instance dim of 4x4x4.
//     
// 
// 
//     let x = insertBits(in, 1u, vidx, 1u)   
// }
