struct OCTree {
    offset: u32,
    mask: u32,
}

@group(0) @binding(0) var<storage, read_write> data: array<OCTree>;

@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {

}
