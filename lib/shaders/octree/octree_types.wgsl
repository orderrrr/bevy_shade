#define_import_path octree

struct OCTreeSettings {
    depth: u32,
    scale: f32,
}

struct OCTreeRuntime {
    depth: u32,
}

struct OCTree {
    mask: u32,
}

struct Voxel {
    col: u32,
    mat: u32,
}

fn get_enclosed_octree(point: vec3<f32>, dim: u32, scale: f32) -> vec3<i32> {
    // Calculate the offset
    let offset = scale / 2.0;
    let adjusted_point = point + vec3<f32>(offset, offset, offset);

    // Adjust point to fit within the range and scale by the dimension
    let scaled_point = adjusted_point / (scale / f32(dim));

    // Use floor to map to the correct grid cell
    return max(min(vec3<i32>(floor(scaled_point)), vec3<i32>(dim - 1)), vec3<i32>(0));
}

fn calc_pos_from_invoc_id(indices: vec3<u32>, i: u32, scale: f32) -> vec3<f32> {
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
