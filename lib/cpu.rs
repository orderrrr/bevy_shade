use std::{fs::File, io::Read};

use bevy_shade_lib::{
    shaders::{octree::settings_plugin::OCTreeSettings, OCTree, Voxel},
    testing::basics::{count_octrees_below, get_enclosed_octree, get_unique_index},
};
use glam::{vec2, vec3, IVec3, UVec2, UVec3, Vec2, Vec3};

const RESOLUTION: u32 = 240;
const MAX_DEPTH: u32 = 3;
const SETTINGS: OCTreeSettings = OCTreeSettings {
    depth: 2,
    scale: 2.0,
};

// fn cube(p: vec3<f32>, d: vec3<f32>) -> f32 {
//     let q = abs(p) - d;
//     return length(max(q, vec3(0.0))) + min(max(q.x, max(q.y, q.z)), 0.);
// }
fn cube(p: Vec3, d: Vec3) -> f32 {
    let q = p.abs() - d;
    // return length(max(q, vec3(0.0))) + min(max(q.x, max(q.y, q.z)), 0.);
    (q.max(Vec3::splat(0.0))).length() + q.x.max(q.y.max(q.z)).min(0.)
}

// fn get_dist(pos: vec3<f32>, i: u32) -> f32 {
//
//     return cube(pos, vec3((settings.scale / f32(1u << i)) / 2.0));
// }
fn get_dist(pos: Vec3, i: u32) -> f32 {
    cube(pos, Vec3::splat((SETTINGS.scale / (1 << i) as f32) / 2.0))
}

// fn calc_pos_from_invoc_id(indices: vec3<u32>, i: u32, scale: f32) -> vec3<f32> {
//     let d = 1u << i;
//     let size: f32 = scale / f32(d);
//
//     let center = (vec3<f32>(indices) + 0.5) * size - (scale / 2.0);
//
//     return center;
// }
fn calc_pos_from_invoc_id(indices: UVec3, i: u32, scale: f32) -> Vec3 {
    let d = 1 << i;
    let size = scale / d as f32;
    (indices.as_vec3() + 0.5) * size - (scale / 2.)
}

// fn get_distance_to_next_octree(gp: vec3<u32>, rp: vec3<f32>, i: u32, scale: f32) -> f32 {
//
//     let index = count_octrees_below(i, settings.depth) + get_unique_index_for_dim(gp, i);
//     let octree = octrees[index];
//     let pos = calc_pos_from_invoc_id(gp, i, settings.scale);
//
//     // get the distance from the old position to the next octree
//     return get_dist(rp - pos, i);
// }
fn get_distance_to_next_octree(
    gp: UVec3,
    rp: Vec3,
    i: u32,
    scale: f32,
    _voxels: &Vec<Voxel>,
    _octrees: &Vec<OCTree>,
) -> f32 {
    // let index = count_octrees_below(i, SETTINGS.depth) + get_unique_index(&gp, i);
    let pos = calc_pos_from_invoc_id(gp, i, scale);
    get_dist(rp - pos, i)
}

// fn get_next_octree_pos(rp: vec3<f32>, rd: vec3<f32>, i: u32, scale: f32) -> vec3<i32> {
//
//     let d = 1u << i;
//
//     // normalize the rd and multiply by the size of the current dim voxel, putting us somewhere inside the next grid
//     let np = rp + (normalize(rd) * (scale / f32(d)));
//
//     // get the next octree in that position.
//     return get_enclosed_octree(np, d, settings.scale); // grid position
// }
fn get_next_octree_pos(rp: Vec3, rd: Vec3, i: u32, scale: f32) -> IVec3 {
    let d = 1 << i;
    let np = rp + (rd * (scale / d as f32));

    get_enclosed_octree(&np, d)
}

// fn get_dist_for_dim(rp: vec3<f32>, rd: vec3<f32>, i: u32) -> f32 {
//     var dist = settings.scale;
//
//     let d = 1u << i;
//
//     var gp = get_enclosed_octree(rp, d, settings.scale); // grid position
//     var gpu = vec3<u32>(gp);
//
//     var ngp = get_next_octree_pos(rp, rd, i, settings.scale);
//     var ngpu = vec3<u32>(ngp);
//
//     let index = count_octrees_below(i, settings.depth) + get_unique_index_for_dim(gpu, i);
//     let octree = octrees[index];
//
//     if octree.mask > 0u {
//
//         dist = get_distance_to_next_octree(gpu, rp, i, settings.scale);
//     }
//
//     if !(octree.mask > 0u) {
//
//         dist = get_distance_to_next_octree(ngpu, rp, i, settings.scale);
//     }
//
//     return dist;
// }
fn get_dist_for_dim(rp: Vec3, rd: Vec3, i: u32, voxels: &Vec<Voxel>, octrees: &Vec<OCTree>) -> f32 {
    eprintln!("i: {}", i);

    let d = 1 << i;
    let mut dist = SETTINGS.scale;

    let gp = get_enclosed_octree(&rp, d);
    let gpu = gp.max(IVec3::splat(0)).as_uvec3();

    let ngp = get_next_octree_pos(rp, rd, i, SETTINGS.scale);
    let ngpu = ngp.max(IVec3::splat(0)).as_uvec3();

    let index = count_octrees_below(i, SETTINGS.depth) + get_unique_index(&gpu, i);
    let octree = octrees[index as usize];

    if octree.mask > 0 && valid_octree_pos(gp, i) {
        dist = get_distance_to_next_octree(gpu, rp, i, SETTINGS.scale, voxels, octrees);
    }

    if !octree.mask > 0 && valid_octree_pos(ngp, i) {
        dist = get_distance_to_next_octree(ngpu, rp, i, SETTINGS.scale, voxels, octrees)
    }

    dist
}

// // currently we only have one octree, i would like to have multiple but for now assuming only one.
// fn valid_octree_pos(gp: vec3<i32>, i: u32) -> bool {
//
//     // make sure it is within the octree bounds.
//     return
//         gp.x > -1 && gp.y > -1 && gp.z > -1 && gp.x < i32((1u << i) - 1) && gp.y < i32((1u << i) - 1) && gp.z < i32((1u << i) - 1);
// }
fn valid_octree_pos(gp: IVec3, i: u32) -> bool {

    let d = 1 << i;
    gp.x > -1 && gp.y > -1 && gp.z > -1 && gp.x < d && gp.y < d && gp.z < d 
}

// fn distance_to_octree(rp: vec3<f32>, rd: vec3<f32>, dim: u32) -> f32 {
//
//     var i = dim;
//     var dist = settings.scale;
//
//     dist = get_dist_for_dim(rp, rd, i);
//
//     if (dist < 0.001) {
//
//         while dist < 0.001 && i <= settings.depth {
//
//             i = i + 1;
//             dist = get_dist_for_dim(rp, rd, i);
//         }
//
//         if (dist < 0.001) {
//             dist = get_dist_for_voxels(rp, rd);
//         }
//     }
//
//
//     return dist;
// }
fn closest_octree(
    rp: Vec3,
    rd: Vec3,
    dim: &mut u32,
    voxel: &Vec<Voxel>,
    octree: &Vec<OCTree>,
) -> f32 {
    let i = dim;
    let mut dist = get_dist_for_dim(rp, rd, *i, voxel, octree);

    if dist < 0.001 {
        while dist < 0.001 && *i <= SETTINGS.depth {
            *i += 1;
            dist = get_dist_for_dim(rp, rd, *i, voxel, octree);
        }

        // if dist < 0.001 {
        //     dist = get_dist_for_voxels(rp, rd);
        // }
    }

    dist
}

// fn cast_ray(ro: vec3<f32>, rd: vec3<f32>) -> f32 {
//
//     var t = 0.;
//
//     // for loop through octree 8next.
//     for (var i: u32 = 0; i < 100; i++) {
//
//         let pos = ro + t * rd;
//         let h = map(pos, rd);
//
//         if h < 0.001 {
//             break;
//         }
//
//         t += h;
//
//         if t > 20. {
//             break;
//         }
//     }
//
//     if t > 20. {
//         t = -1.;
//     }
//
//     return t;
// }

pub fn cast_ray(ro: Vec3, rd: Vec3, voxel: &Vec<Voxel>, octree: &Vec<OCTree>) -> f32 {
    let mut t = 0.0;
    let mut depth = 0;

    for _ in 0..200 {
        let pos = ro + t * rd;

        let d = closest_octree(pos, rd, &mut depth, voxel, octree);

        // if d < 0.001 && depth < MAX_DEPTH {
        //
        // }

        if d < 0.001 {
            break;
        }

        t += d;

        if t > 20. {
            break;
        }
    }

    if t > 20. {
        t = -1.;
    }

    t
}

fn calc_normal(pos: Vec3, voxel: &Vec<Voxel>, octree: &Vec<OCTree>) -> Vec3 {
    let e = vec2(0.0001, 0.0);
    let mut d = MAX_DEPTH;

    return (vec3(
        closest_octree(
            pos + vec3(e.x, e.y, e.y),
            vec3(0.0, 0.0, 0.0),
            &mut d,
            voxel,
            octree,
        ) - closest_octree(
            pos - vec3(e.x, e.y, e.y),
            vec3(0.0, 0.0, 0.0),
            &mut d,
            voxel,
            octree,
        ),
        closest_octree(
            pos + vec3(e.y, e.x, e.y),
            vec3(0.0, 0.0, 0.0),
            &mut d,
            voxel,
            octree,
        ) - closest_octree(
            pos - vec3(e.y, e.x, e.y),
            vec3(0.0, 0.0, 0.0),
            &mut d,
            voxel,
            octree,
        ),
        closest_octree(
            pos + vec3(e.y, e.y, e.x),
            vec3(0.0, 0.0, 0.0),
            &mut d,
            voxel,
            octree,
        ) - closest_octree(
            pos - vec3(e.y, e.y, e.x),
            vec3(0.0, 0.0, 0.0),
            &mut d,
            voxel,
            octree,
        ),
    ))
    .normalize();
}

// fn render(ro: vec3<f32>, rd: vec3<f32>, col: ptr<function, vec3<f32>>) {
//     let t: f32 = cast_ray(ro, rd);
//
//     var pos = vec3<f32>(0.0);
//
//     // if its -1 leave blank and use sky color.
//     if t > 0. {
//
//         let pos = ro + t * rd;
//         let nor = calc_normal(pos);
//         let mate = vec3(0.18);
//
//         let sun_dir = normalize(vec3(.8, .4, .4));
//         let sun_dif = clamp(dot(nor, sun_dir), 0., 1.);
//         let sun_sha = step(cast_ray(pos + nor * 0.001, sun_dir), 0.);
//
//         let sky_dif = clamp(.5 + .5 * dot(nor, vec3(0.1, 1.0, 0.)), 0., 1.);
//         let bou_dif = clamp(.5 + .5 * dot(nor, vec3(0., -1., 0.)), .0, 1.);
//
//         *col = mate * vec3(7., 4.5, 3.) * sun_dif * sun_sha;
//         *col += mate * vec3(0.5, 0.8, 0.9) * sky_dif;
//         *col += mate * vec3(1., 0.4, 0.3) * bou_dif;
//     }
// }
fn render(ro: Vec3, rd: Vec3, voxel: &Vec<Voxel>, octree: &Vec<OCTree>) -> Vec3 {
    let t = cast_ray(ro, rd, voxel, octree);

    if t > 0. {
        let pos = ro + t * rd;
        return calc_normal(pos, voxel, octree);
    }

    Vec3::new(0.0, 0.0, 0.0)
}

fn fragment(pos: UVec2, voxel: &Vec<Voxel>, octree: &Vec<OCTree>) -> Vec3 {
    // custom uv, not quite the same as in.uv.
    let r = Vec2::new(RESOLUTION as f32, RESOLUTION as f32);
    let uv: Vec2 = ((pos.as_vec2() * 2.) - r) / r.y;

    // rotation around 0.,,
    let ro = Vec3::new(0.0, 0.0, -2.);

    // todo convert this from linear algebra rotation to geometric algebra.
    let ta = Vec3::new(0., 0., 0.);
    let ww = (ta - ro).normalize();
    let uu = (ww.cross(Vec3::new(0.0, 1.0, 0.0))).normalize();
    let vv = (uu.cross(ww)).normalize();

    let rd: Vec3 = (uv.x * uu + uv.y * vv + 2.0 * ww).normalize();

    render(ro, rd, voxel, octree)
}

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

    (0..RESOLUTION)
        .into_iter()
        .map(|x| {
            (0..RESOLUTION)
                .into_iter()
                .map(move |y| UVec2::new(x.clone(), y.clone()))
        })
        .flatten()
        .map(|pos| fragment(pos, &voxels, &octrees))
        .for_each(|col| {
            let ir = (255.99 * col.x) as u8;
            let ig = (255.99 * col.y) as u8;
            let ib = (255.99 * col.z) as u8;

            println!("{} {} {}", ir, ig, ib);
        });
}
