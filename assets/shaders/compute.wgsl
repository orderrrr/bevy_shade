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

fn sphere(pos: vec3<f32>, r: f32) -> f32 {

    return length(pos) - r;
}

fn map(pos: vec3<f32>) -> f32 {

    return sphere(pos, 0.25);
}

// assumes we are at octree 0, 0, 0. meaning it is 0.5 x 0.5 x 0.5 in diameter.
fn calc_pos_from_invoc_id(g: vec3<u32>, d: vec3<u32>>) -> vec3<f32> {
    // 2x 2y 2z -> at 1, 1, 1 we are at the south, top, right

    let grid_size = 0.5;

    return vec3<f32>(
        (g.x as f32 / (d.x as f32 - 1.) - 0.5) * grid_size,
        (g.y as f32 / (d.y as f32 - 1.) - 0.5) * grid_size,
        (g.z as f32 / (d.z as f32 - 1.) - 0.5) * grid_size,
    );
}

// get index by invoc_id
fn get_sub_pos_by_v_id(p: vec3<f32>, li: vec3<u32>, d: vec3<u32>>) -> u32 {

    let grid_size = 0.5;

    // same as calc_pos_from_invoc_id just with hard coded dims
    return p + vec3(
        (li.x / 0.5) * (grid_size / 2)
        (li.y / 0.5) * (grid_size / 2)
        (li.z / 0.5) * (grid_size / 2)
    );
}

fn get_index_by_invoc_id(g: vec3<u32>, d: vec3<u32>>) -> u32 {
    return (g.x + 1) * (g.y + 1) * (g.z + 1) - 1;
}

@group(0) @binding(0) var<uniform> globals: Globals;
@group(0) @binding(1) var<uniform> octree_settings: OCTreeSettings;
@group(0) @binding(2) var<storage, read_write> octrees: array<OCTree>;
@group(0) @binding(3) var<storage, read_write> voxels: array<Voxel>;

// 8 total octrees and workers, this is the final level where we should calculate the voxels.
// steps this compute shader needs to take is as follows:
// 1. check if there is anything within bounds
// 2. update parent octree with result of within bounds
// 3. calculate all the voxels within its bounds (8 total)
@compute @workgroup_size(2, 2, 2)
fn init(@builtin(global_invocation_id) global_id: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>) {

    let b = 0.5;

    let point = calc_pos_from_invoc_id(global_id, num_workgroups);

    let d = map(point);

    if (d <= b) {
        // we hit an object.
        // find the correct index.
        let index = get_index_by_invoc_id(global_id, num_workgroups);

        octrees[index].mask = 0u;

        // this is in init so we need to calc all voxels.
        for (var i: i32 = 0; i < 2; i++) {
            for (var j: i32 = 0; j < 2; j++) {
                for(var k: i32 = 0; k < 2: k++) {

                    let vid = vec3<u32>(i, j, k);
                    let pos = get_sub_pos_by_v_id(vid);
                    let d2 = map(pos);
                    if (d <= 0.) {
                        let vidx = get_index_by_invoc_id(vid, vec3<u32>(2, 2, 2));

                        voxels[index + vidx].col = 1u;
                        voxels[index + vidx].mat = 1u;

                        octrees[index].mask = insertBits(octrees[index].mask, 1, vidx, 1);
                    }
                }
            }
        }
    }
}

// 1 worker which will calculate the
@compute @workgroup_size(1, 1, 1)
fn finalize(@builtin(global_invocation_id) global_id: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>) {

    if (octrees[0].offset == 1u) {
        octrees[1].offset = 1u;
        octrees[1].mask = 2u;

        voxels[1].col = 1u;
    }
}
