use bevy::{
    ecs::prelude::*,
    render::{
        extract_component::ExtractComponent, extract_resource::ExtractResource, render_resource::*,
    },
};
use bytemuck::{Pod, Zeroable};
use zerocopy::{FromBytes, FromZeroes};

pub mod compute;
pub mod fragment;
pub mod lib;

#[repr(C)]
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    ShaderType,
    ExtractResource,
    Resource,
    Pod,
    Zeroable,
    FromBytes,
    FromZeroes,
)]
pub struct Voxel {
    col: u32, // TODO - 64 bit colors.
    mat: u32, // TODO, maybe multiple mats here.
}

#[repr(C)]
#[derive(
    Copy,
    Debug,
    Default,
    Clone,
    ShaderType,
    ExtractResource,
    Resource,
    Pod,
    Zeroable,
    FromBytes,
    FromZeroes,
)]
pub struct OCTree {
    offset: u32, // array offset in voxey/octree
    mask: u32,
}

#[repr(C)]
#[derive(Component, Default, Clone, Copy, ExtractComponent, ShaderType)]
pub struct OCTreeSettings {
    pub depth: u32,
    pub scale: f32,
}

#[repr(C)]
#[derive(
    Copy,
    Debug,
    Default,
    Clone,
    ShaderType,
    ExtractResource,
    Resource,
    Pod,
    Zeroable,
    FromBytes,
    FromZeroes,
)]
pub struct OCTreeRuntimeData {
    pub current_depth: u32,
}

impl OCTreeRuntimeData {
    pub fn new(current_depth: u32) -> OCTreeRuntimeData {
        OCTreeRuntimeData { current_depth }
    }
}

// #[derive(Resource, AsBindGroup)]
// pub struct OCTreeData {
//     // voxels: BufferVec<Voxel>,
//     octree: BufferVec<OCTree>,
// }
//
// impl Default for OCTreeData {
//     fn default() -> Self {
//         let mut voxels = BufferVec::new(BufferUsages::STORAGE);
//         let mut octree = BufferVec::new(BufferUsages::STORAGE);
//         octree.push(OCTree::default());
//         voxels.push(Voxel::default());
//
//         OCTreeData { voxels, octree }
//     }
// }
