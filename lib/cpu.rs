use std::{fs::File, io::Read};
use rayon::prelude::*;


use bevy_shade_lib::{testing::octree::{RESOLUTION, fragment}, shaders::{OCTree, Voxel}};
use glam::{UVec2, Vec3};
use rayon::iter::{IntoParallelIterator, ParallelBridge};

fn main() {
    println!("P3\n{} {}\n255", RESOLUTION, RESOLUTION);

    let voxels: Vec<Voxel> = {
        let mut voxels_file = File::open("voxels.json").unwrap();
        let mut slice: Vec<u8> = vec![];
        let _ = voxels_file.read_to_end(&mut slice);

        serde_json::from_slice(&slice).unwrap()
    };

    let octrees: Vec<OCTree> = {
        let mut voxels_file = File::open("octrees.json").unwrap();
        let mut slice: Vec<u8> = vec![];
        let _ = voxels_file.read_to_end(&mut slice);

        serde_json::from_slice(&slice).unwrap()
    };

    let img: Vec<UVec2> = (0..RESOLUTION)
        .into_iter()
        .map(|x| {
            (0..RESOLUTION)
                .into_iter()
                .map(move |y| UVec2::new(x.clone(), y.clone()))
        })
        .flatten().collect();

    let res: Vec<Vec3> = img
        .into_par_iter()
        .map(|pos| fragment(pos, &voxels, &octrees))
        // .rev()
        .collect();

    res
        .into_iter()
        .for_each(|col| {
        let ir = (255.99 * col.x) as u8;
        let ig = (255.99 * col.y) as u8;
        let ib = (255.99 * col.z) as u8;

        println!("{} {} {}", ir, ig, ib);
    })
}
