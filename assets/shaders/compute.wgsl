struct OCTree {
    @location(0) offset: u32,
    @location(1) mask: u32,
}

@group(0) @binding(0) var<storage, read_write> data: array<OCTree>;

@compute @workgroup_size(128)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    data[0].offset = 1u;
    data[0].mask = 2u;
}
