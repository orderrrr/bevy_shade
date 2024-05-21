#import bevy_render::globals::Globals

struct OCTreeSettings {
    depth: u32,
    scale: f32,
}

struct OCTree {
    @location(0) offset: u32,
    @location(1) mask: u32,
}

struct Voxel {
    @location(0) col: u32,
    @location(1) mat: u32,
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

    return sphere(pos, 0.25);
}

fn calc_pos_from_invoc_id(block_indices: vec3<u32>, depth: u32) -> vec3<f32> {
    // Calculate the size of a block at the given depth level in the octree
    // Divide the overall scale (settings.scale) by 2 raised to the power of depth
    let block_size = settings.scale / pow(2.0, f32(depth));

    // Calculate the offset from the corner of a block to its center
    // Divide the block size by 2 to get half of the block size
    let center_offset = block_size / 2.0;

    // Calculate and return the position of the block based on the block indices and depth
    // Multiply the converted block_indices by block_size to scale the indices based on the block size at the given depth
    // Subtract settings.scale / 2.0 from the scaled indices to offset the position towards the origin of the octree
    // Add the center_offset to the result to get the final position of the block's center
    return vec3<f32>(block_indices) * block_size - (settings.scale / 2.0) + center_offset;
}

fn calp_vpos_from_vid_and_parent(parent_depth: u32, parent_indices: vec3<u32>, child_offset: vec3<u32>) -> vec3<f32> {
    // Calculate the size of the parent block at the given parent_depth level in the octree
    // Divide the overall scale (settings.scale) by 2 raised to the power of parent_depth
    let parent_block_size = settings.scale / pow(2.0, f32(parent_depth));

    // Calculate the size of the child block based on the parent block size
    // Divide the parent block size by 2 to get half of the parent block size
    let child_block_size = parent_block_size / 2.0;

    // Calculate the offset from the corner of the child block to its center
    // Divide the child block size by 2 to get half of the child block size
    let child_center_offset = child_block_size / 2.0;

    // Calculate the position of the parent block's center by calling the calc_pos_from_invoc_id function
    // Pass the parent_indices and parent_depth as arguments to calculate the parent block's center position
    let parent_center = calc_pos_from_invoc_id(parent_indices, parent_depth);

    // Calculate and return the position of the child block based on the parent block's center position and the child offset
    // Multiply the converted child_offset vector by child_block_size to calculate the offset of the child block within the parent block
    // Add the calculated offset to the parent_center to get the position of the child block
    // Subtract the child_center_offset from the result to get the final position of the child block's center
    return parent_center + vec3<f32>(child_offset) * child_block_size - child_center_offset;
}

fn get_unique_index_for_dim(g: vec3<u32>, d: u32) -> u32 {
    return g.x + g.y * d + g.z * d * d;
}

fn get_child_index(parent_pos: vec3<u32>, child_local_pos: vec3<u32>, depth: u32) -> u32 {
    let child_pos = parent_pos * 2 + child_local_pos;
    let child_depth_dim = 1u << depth; // Calculate the dimensions of the grid at the child's depth
    return get_unique_index_for_dim(child_pos, child_depth_dim);
}

fn get_position_from_unique_index(index: u32, d: u32) -> vec3<u32> {
    let z = index / (d * d);
    let remaining = index % (d * d);
    let y = remaining / d;
    let x = remaining % d;
    return vec3<u32>(x, y, z);
}

fn count_octrees_below(cd: u32, d: u32) -> u32 {
    return u32(pow(8.0, f32(d + 1u)) / 7.0 - pow(8.0, f32(cd + 1u)) / 7.0);
}

fn init_with_dims(global_id: vec3<u32>, num_workgroups: vec3<u32>) {

    let b = settings.scale;

    let point = calc_pos_from_invoc_id(global_id, num_workgroups.x - 1);

    let d = map(point);

    if (d <= b) {
    // if (true) {
        // we hit an object.
        // find the correct index.
        let index = get_unique_index_for_dim(global_id, num_workgroups.x);

        var mask = 0u;

        // this is in init so we need to calc all voxels.
        for (var i: u32 = 0; i < 2; i++) {
            for (var j: u32 = 0; j < 2; j++) {
                for(var k: u32 = 0; k < 2; k++) {

                    let vid = vec3<u32>(i, j, k);
                    let pos = calp_vpos_from_vid_and_parent(num_workgroups.x - 1, global_id, vid);

                    let d2 = map(pos);

                    if (d2 <= 0.) {
                    // if (true) {

                        let vidx = get_unique_index_for_dim(vid, u32(2));

                        voxels[index + vidx].col = 1u;
                        voxels[index + vidx].mat = 1u;

                        mask = insertBits(mask, 1u, vidx, 1u);
                    }
                }
            }
        }

        octrees[index].mask = mask;
    }
}

fn finalize_with_dims(global_id: vec3<u32>, num_workgroups: vec3<u32>) {

    let index = count_octrees_below(num_workgroups.x - 1, settings.depth) + get_unique_index_for_dim(global_id, num_workgroups.x);

    let b = 0.5;

    let point = calc_pos_from_invoc_id(global_id, num_workgroups.x - 1);
    let d = map(point);

    if (d <= b) {
    // if (true) {

        var mask = 0u;

        for (var i: u32 = 0; i < 2; i++) {
            for (var j: u32 = 0; j < 2; j++) {
                for(var k: u32 = 0; k < 2; k++) {

                    let vid = vec3<u32>(i, j, k);
                    let ci = count_octrees_below(num_workgroups.x, settings.depth) + get_child_index(global_id, vid, num_workgroups.x);

                    if octrees[ci].mask > 1 {
                        let vidx = get_unique_index_for_dim(vid, u32(2));

                        mask = insertBits(mask, 1u, vidx, 1u);
                    }
                }
            }
        }

        octrees[index].mask = mask;
    }
}
