use bevy::{
    ecs::prelude::*,
    render::{extract_resource::ExtractResource, render_resource::*},
};
use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};
use zerocopy::{FromBytes, FromZeroes};

pub mod compute;
pub mod fragment;
pub mod octree;

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
    Serialize,
    Deserialize,
)]
pub struct Voxel {
    pub col: u32, // TODO - 64 bit colors.
    pub mat: u32, // TODO, maybe multiple mats here.
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
    Serialize,
    Deserialize,
)]
pub struct OCTree {
    pub mask: u32,
}
