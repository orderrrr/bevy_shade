use core::panic;

use crate::{
    shaders::{octree::settings_plugin::OCTreeSettings, OCTree, Voxel},
    testing::basics::{count_octrees_below, get_enclosed_octree, get_unique_index},
};
use glam::{ivec3, vec2, vec3, IVec3, UVec2, Vec2, Vec2Swizzles, Vec3};

pub const RESOLUTION: u32 = 512;
pub const SETTINGS: OCTreeSettings = OCTreeSettings {
    depth: 2,
    scale: 2.0,
};
pub const MIN_BOUND: Vec3 = vec3(-1.0, -1.0, -1.0);
pub const MAX_BOUND: Vec3 = vec3(1.0, 1.0, 1.0);

pub fn octree_size(i: u32, scale: f32) -> f32 {
    scale / (1 << i) as f32
}

pub fn cube(p: Vec3, d: Vec3) -> f32 {
    let q = p.abs() - d;
    (q.max(Vec3::splat(0.0))).length() + q.x.max(q.y.max(q.z)).min(0.)
}

pub fn get_dist(pos: Vec3, i: u32) -> f32 {
    cube(pos, Vec3::splat(octree_size(i, SETTINGS.scale)) / 2.0)
}

pub fn calc_pos_from_invoc_id(indices: IVec3, i: u32, scale: f32) -> Vec3 {
    let size = octree_size(i, SETTINGS.scale);
    (indices.as_vec3() + 0.5) * size - (scale / 2.)
}

pub fn get_distance_to_next_octree(
    gp: IVec3,
    rp: Vec3,
    i: u32,
    scale: f32,
    _voxels: &Vec<Voxel>,
    _octrees: &Vec<OCTree>,
) -> f32 {
    // let index = count_octrees_below(i, SETTINGS.depth) + get_unique_index(&gp, i);
    let pos = calc_pos_from_invoc_id(gp, i, scale);
    //eprintln!("rp: {}, cube_pos: {}", rp, pos);
    get_dist(rp - pos, i)
}

pub fn get_next_octree_pos(rp: Vec3, rd: Vec3, i: u32, scale: f32) -> IVec3 {
    let np = rp + (rd * (octree_size(i, scale)) / 8.0);

    get_enclosed_octree(np, 1 << i, SETTINGS.scale)
}

pub fn offset_pos(rp: Vec3, rd: Vec3, i: u32, scale: f32) -> Vec3 {
    let d = 1 << i;
    rp + (rd * (scale / d as f32))
}

pub fn get_next_grid(rp: Vec3, rd: Vec3, i: u32, offset: Vec3) -> IVec3 {
    let gp = get_enclosed_octree(rp, 1 << i, SETTINGS.scale);

    let box_dims = octree_size(i, SETTINGS.scale) / 2.0;

    let min = offset - box_dims;
    let max = offset + box_dims;

    let t1 = (min - rp) / rd;
    let t2 = (max - rp) / rd;

    let tnearx = t1.x.min(t2.x);
    let tfarx = t1.x.max(t2.x);

    let tneary = t1.y.min(t2.y);
    let tfary = t1.y.max(t2.y);

    let tnearz = t1.z.min(t2.z);
    let tfarz = t1.z.max(t2.z);

    let tnear = tnearx.max(tneary.max(tnearz));
    let tfar = tfarx.min(tfary.min(tfarz));

    if tnear > tfar || tfar < 0. {
        // panic!("NO INTERSECT, how?");
    }

    let mins = vec3(tnearx, tneary, tnearz);
    let maxs = vec3(tfarx, tfary, tfarz);

    let min_value = mins.max_element();

    let max_value = maxs.min_element();

    let min = min_value.min(max_value);

    ivec3(
        if mins.x == min_value {
            gp.x + (mins.x.signum() as i32)
        } else {
            gp.x
        },
        if mins.y == min_value {
            gp.y + (mins.y.signum() as i32)
        } else {
            gp.y
        },
        if mins.z == min_value {
            gp.z + (mins.z.signum() as i32)
        } else {
            gp.z
        },
    )
}

pub fn move_to_edge(rp: Vec3, rd: Vec3, i: u32, offset: Vec3) -> f32 {
    // float tx1, tx2, ty1, ty2, tz1, tz2, tNear, tFar;
    // tx1 = (xMin - posn.x) / dir.x;
    // tx2 = (xMax - posn.x) / dir.x;
    // ty1 = (yMin - posn.y) / dir.y;
    // ty2 = (yMax - posn.y) / dir.y;
    // tz1 = (zMin - posn.z) / dir.z;
    // tz2 = (zMax - posn.z) / dir.z;
    //
    // tNear = std::max(std::min(tx1, tx2), std::max(std::min(ty1, ty2), std::min(tz1, tz2)));
    // tFar = std::min(std::max(tx1, tx2), std::min(std::max(ty1, ty2), std::max(tz1, tz2)));
    //
    // if (tNear > tFar || tFar < 0) {
    //     return -1;
    // }
    //
    // return tNear;

    let box_dims = octree_size(i, SETTINGS.scale) / 2.0;

    let min = offset - box_dims;
    let max = offset + box_dims;

    let t1 = (min - rp) / rd;
    let t2 = (max - rp) / rd;

    // let tnear = std::max(std::min(tx1, tx2), std::max(std::min(ty1, ty2), std::min(tz1, tz2)));

    // let tnear = min.x.max(t_min.y.max(t_min.z));

    let tnear = t1.x.min(t2.x).max(t1.y.min(t2.y).max(t1.z.min(t2.z)));
    let tfar = t1.x.max(t2.x).min(t1.y.max(t2.y).min(t1.z.max(t2.z)));

    if tnear > tfar || tfar < 0. {
        panic!("NO INTERSECT, how?");
    }

    tnear.abs()
}

pub fn get_current_octree_dist(
    rp: Vec3,
    i: u32,
    voxels: &Vec<Voxel>,
    octrees: &Vec<OCTree>,
) -> f32 {
    //First we need to check if the octree we are currently in is empty.
    // let gp = get_enclosed_octree(&rp, d, SETTINGS.scale);
    let gp = IVec3::splat(0);

    let dist = get_distance_to_next_octree(gp, rp, i, SETTINGS.scale, voxels, octrees);

    // return rp.length() - 1.0;
    return dist;

    // return cube(rp, Vec3::splat(SETTINGS.scale / (1 << i) as f32 / 2.0));
}

pub fn get_dist_for_dim(
    rp: Vec3,
    rd: Vec3,
    i: u32,
    t: &mut f32,
    voxels: &Vec<Voxel>,
    octrees: &Vec<OCTree>,
) -> Vec2 {
    let mut pos = rp.clone();

    let d = 1 << i;
    let below = count_octrees_below(i, SETTINGS.depth);

    // current ray
    let gp = get_enclosed_octree(pos, d, SETTINGS.scale);

    eprintln!("GP: {}", gp);

    let cube_offset = calc_pos_from_invoc_id(gp, i, SETTINGS.scale);
    // next ray
    let mut gp = get_next_grid(pos, rd, i, cube_offset);

    eprintln!("GP NEW: {}", gp);

    for _ in 0..300 {
        eprintln!("GP: {}", gp);

        let gpu = gp.max(IVec3::splat(0)).as_uvec3();

        if valid_octree_pos(gp, i) {
            let index = below + get_unique_index(&gpu, i);
            let octree = octrees[index as usize];

            if octree.mask > 0 {
                //eprintln!("OCTREE HAS MASK");
                let dist = get_distance_to_next_octree(gp, rp, i, SETTINGS.scale, voxels, octrees);
                let other_dist =
                    get_distance_to_next_octree(gp, pos, i, SETTINGS.scale, voxels, octrees);
                //eprintln!("DIST: {}", dist);
                if other_dist < 0.001 {
                    return Vec2::new(0.0001, 1.0);
                }

                return Vec2::new(dist, 0.0);
            }
        }

        let cube_offset = calc_pos_from_invoc_id(gp, i, SETTINGS.scale);
        gp = get_next_grid(pos, rd, i, cube_offset);

        let dist = get_distance_to_next_octree(gp, pos, i, SETTINGS.scale, voxels, octrees);

        *t = *t + dist;
        pos = pos + *t * rd;
    }

    return vec2(-1.0, 0.0);
}

pub fn valid_octree_pos(gp: IVec3, i: u32) -> bool {
    let d = 1 << i;
    gp.x > -1 && gp.y > -1 && gp.z > -1 && gp.x < d && gp.y < d && gp.z < d
}

pub fn closest_octree(
    rp: Vec3,
    rd: Vec3,
    dim: &mut u32,
    t: &mut f32,
    voxel: &Vec<Voxel>,
    octree: &Vec<OCTree>,
) -> f32 {
    let i = dim.clone();

    let dist = get_dist_for_dim(rp, rd, i, t, voxel, octree);

    if dist.y > 0.0 {
        *dim = 1;
    }

    dist.x

    // return get_current_octree_dist(rp, *dim, voxel, octree);
}

pub fn cast_ray(ro: Vec3, rd: Vec3, voxel: &Vec<Voxel>, octree: &Vec<OCTree>) -> f32 {
    let mut t = 0.0;
    let mut depth = 0;

    for _ in 0..300 {
        let pos = ro + t * rd;

        //eprintln!("POS IN RAY: {}", pos);

        let d = closest_octree(pos, rd, &mut depth, &mut t, voxel, octree);
        // let d = get_current_octree_dist(pos, depth, voxel, octree);
        //eprintln!("d: {}, t: {}", d, t);

        //eprintln!("d: {}", d);

        // if d < 0.001 && depth < MAX_DEPTH {
        //
        // }

        // if d < 0.001 && depth > 0 {
        if d < 0.001 {
            break;
        }

        t += d;

        if t > 20. {
            break;
        }
    }

    if t > 20. {
        // if t > 20. || depth == 0 {
        t = -1.;
    }

    //eprintln!("t is: {}", t);

    // assert_eq!(depth, 2);

    t
}

pub fn calc_normal(pos: Vec3, voxel: &Vec<Voxel>, octree: &Vec<OCTree>) -> Vec3 {
    let e = vec2(0.001, 0.0);
    let d = 0;

    return (vec3(
        get_current_octree_dist(pos + e.xyy(), d, voxel, octree)
            - get_current_octree_dist(pos - e.xyy(), d, voxel, octree),
        get_current_octree_dist(pos + e.yxy(), d, voxel, octree)
            - get_current_octree_dist(pos - e.yxy(), d, voxel, octree),
        get_current_octree_dist(pos + e.yyx(), d, voxel, octree)
            - get_current_octree_dist(pos - e.yyx(), d, voxel, octree),
    ))
    .normalize();
}

pub fn render(ro: Vec3, rd: Vec3, voxel: &Vec<Voxel>, octree: &Vec<OCTree>) -> Vec3 {
    let t = cast_ray(ro, rd, voxel, octree);

    if t > 0. {
        let pos = ro + (t * rd);
        return vec3(1.0, 1.0, 1.0);
        // return calc_normal(pos, voxel, octree);
    }

    Vec3::new(0.0, 0.0, 0.0)
}

pub fn fragment(pos: UVec2, voxel: &Vec<Voxel>, octree: &Vec<OCTree>) -> Vec3 {
    // custom uv, not quite the same as in.uv.
    let r = Vec2::splat(RESOLUTION as f32);
    // let uv: Vec2 = (pos.as_vec2()) / r;
    let uv: Vec2 = ((pos.as_vec2() * 2.) - r) / r.y;
    // uv = uv * vec2(1.0, -1.0);

    let t = 7.;

    // rotation around 0.,,
    let ro = Vec3::new(5. * f32::sin(t), 0., 5. * f32::cos(t));

    // todo convert this from linear algebra rotation to geometric algebra.
    let ta = Vec3::new(0., 0., 0.);
    let ww = (ta - ro).normalize();
    let uu = (ww.cross(Vec3::new(0.0, 1.0, 0.0))).normalize();
    let vv = (uu.cross(ww)).normalize();

    let rd: Vec3 = (uv.x * uu + uv.y * vv + 2.0 * ww).normalize();

    let render = render(ro, rd, voxel, octree);

    if pos.as_vec2() == Vec2::new(57.0, 42.0) {
        //eprintln!("POS: {}", pos);
        //eprintln!("UV: {}", uv);
        //eprintln!("{}", render);
    }

    render

    // return uv.extend(0.0);
}

#[cfg(test)]
mod ray_test {
    use super::{closest_octree, RESOLUTION};
    use crate::shaders::{OCTree, Voxel};
    use glam::{Vec2, Vec3};
    use std::{fs::File, io::Read};

    pub fn cast_ray_hit_full_depth(
        ro: Vec3,
        rd: Vec3,
        voxel: &Vec<Voxel>,
        octree: &Vec<OCTree>,
    ) -> f32 {
        let mut t = 0.0;
        let mut depth = 0;

        for _ in 0..300 {
            let pos = ro + t * rd;

            //eprintln!("POS IN RAY: {}", pos);

            let d = closest_octree(pos, rd, &mut depth, &mut t, voxel, octree);
            // let d = get_current_octree_dist(pos, depth, voxel, octree);
            //eprintln!("d: {}, t: {}", d, t);

            //eprintln!("d: {}", d);

            // if d < 0.001 && depth < MAX_DEPTH {
            //
            // }

            // if d < 0.001 && depth > 0 {
            if d < 0.001 {
                break;
            }

            t += d;

            if t > 20. {
                break;
            }
        }

        if t > 20. {
            // if t > 20. || depth == 0 {
            t = -1.;
        }

        //eprintln!("t is: {}", t);

        // assert_eq!(depth, 2);

        t
    }

    #[test]
    fn test_single_ray() {
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

        // custom uv, not quite the same as in.uv.
        let r = Vec2::splat(RESOLUTION as f32);
        let pos = r / 2.0;
        let uv: Vec2 = ((pos * 2.) - r) / r.y;

        let t = 7.;

        // rotation around 0.,,
        let ro = Vec3::new(5. * f32::sin(t), 0., 5. * f32::cos(t));

        // todo convert this from linear algebra rotation to geometric algebra.
        let ta = Vec3::new(0., 0., 0.);
        let ww = (ta - ro).normalize();
        let uu = (ww.cross(Vec3::new(0.0, 1.0, 0.0))).normalize();
        let vv = (uu.cross(ww)).normalize();

        let rd: Vec3 = (uv.x * uu + uv.y * vv + 2.0 * ww).normalize();

        //eprintln!("POS: {}", pos);
        //eprintln!("UV: {}", uv);

        assert_eq!(cast_ray_hit_full_depth(ro, rd, &voxels, &octrees), -1.0);
    }
}
