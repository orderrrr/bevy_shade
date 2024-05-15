use bevy::{ecs::system::Resource, math::Vec4, render::{extract_resource::ExtractResource, render_resource::AsBindGroup}};

mod compute;
mod fragment;

#[derive(Resource, Clone, Deref, ExtractResource, AsBindGroup)]
pub struct Voxel {
    col: Vec4,
    mat: i8, // TODO, maybe multiple mats here.
}

#[derive(Resource, Clone, Deref, ExtractResource, AsBindGroup)]
pub struct OCTree {
    data: vec<i32>, // indexes
    mask: i16,
}

#[derive(Resource, Clone, Deref, ExtractResource, AsBindGroup)]
pub struct OCTreeData {
    voxels: vec<Voxel>,
    octree: vec<OCTree>,
}


#[derive(Resource, Clone, Deref, ExtractResource, AsBindGroup)]
pub struct OCTreeDataBuffers {
    buff00: OCTreeData,
    buff01: OCTreeData,
}
