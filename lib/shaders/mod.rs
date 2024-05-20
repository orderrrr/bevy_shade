use bevy::{
    ecs::prelude::*,
    render::{extract_resource::ExtractResource, render_resource::*},
};
use bytemuck::{Pod, Zeroable};
use zerocopy::{FromBytes, FromZeroes};

pub mod compute;
pub mod fragment;

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
