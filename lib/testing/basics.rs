use glam::{vec3, IVec3, UVec3, Vec3};

use super::octree::{octree_size, SETTINGS};

#[allow(dead_code)]
pub fn get_child_pos(parent_pos: &UVec3, child_rel_pos: &UVec3) -> UVec3 {
    *parent_pos * 2 + *child_rel_pos
}

#[allow(dead_code)]
pub fn get_unique_index(pos: &UVec3, i: u32) -> u32 {
    let d = 1 << i;
    pos.x + pos.y * d + pos.z * d * d
}

#[allow(dead_code)]
pub fn get_pos_from_grid_pos(pos: &UVec3, i: u32, scale: f32) -> Vec3 {
    let s = scale as f32;
    let d = (1 << i) as f32;
    let scale: f32 = s / d;
    let offset: f32 = if i == 0 { 0.0 } else { scale * 0.5 };

    return (pos.as_vec3() * offset) - offset;
}

#[allow(dead_code)]
pub fn count_octrees_below(cd: u32, i: u32) -> u32 {
    ((8_u32.pow(i + 1) as f64) / 7.0 - (8_u32.pow(cd + 1) as f64) / 7.0) as u32
}

#[allow(dead_code)]
pub fn get_enclosed_octree(point: Vec3, dim: usize, scale: f32) -> IVec3 {
    let offset = scale / 2.0;
    let scale = scale / dim as f32;
    return ((point.clone() + offset) / scale).floor().as_ivec3();
}

pub fn signf(f: f32) -> f32 {
    if f == 0. {
        return 0.;
    } else {
        return f.signum();
    }
}

pub fn sign(p: Vec3) -> Vec3 {
    vec3(signf(p.x), signf(p.y), signf(p.z))
}

pub fn mask_pos(p: Vec3) -> Vec3 {
    ((sign(p) + 1.) / 2.).floor()
}

pub fn mask_neg(p: Vec3) -> Vec3 {
    ((sign(p) - 1.) / -2.).floor()
}

// pub fn get_next_grid_y(rp: Vec3, rd: Vec3, i: u32) -> Vec3 {
//     let box_dims = octree_size(i, SETTINGS.scale) / 2.0;
//
//     let gp = get_enclosed_octree(rp, 1 << i, SETTINGS.scale).as_vec3();
//     // basically if rd is moving positive in any directions, we increase by one on those directions.
//     let ay = ((gp + mask_pos(rd))) * box_dims;
//
//     let ang = (rd.dot(vec3(1.0, 1.0, 1.0).normalize()) / rd.length()).acos();
//     // let ya = rd.y.signum() * box_dims;
//
//     let ax = rp.x + (rp.y + ay.y) / ang.tan();
//     let az = rp.z + (rp.x + ay.x) / ang.tan();
//
//     eprintln!("box_dims - {}", box_dims);
//     eprintln!("gp - {}", gp);
//     eprintln!("ay - {}", ay);
//     eprintln!("ang - {}", ang);
//     eprintln!("tan - {}", ang.tan());
//     eprintln!("ax - {}", ax);
//     eprintln!("az - {}", ax);
//
//     vec3(ay.y, ax, az)
// }

pub fn get_next_grid_y(rp: Vec3, rd: Vec3, i: u32) -> Vec3 {
    let box_dims = octree_size(i, SETTINGS.scale) / 2.0;

    let gp = get_enclosed_octree(rp, 1 << i, SETTINGS.scale).as_vec3();
    // basically if rd is moving positive in any directions, we increase by one on those directions.
    let ay = (gp + mask_pos(rd)) * box_dims;

    let ang = (rd.dot(vec3(1.0, 1.0, 1.0).normalize()) / rd.length()).acos();
    // let ya = rd.y.signum() * box_dims;

    let ax = rp.x + (rp.y + ay.y) / ang.tan();
    let az = rp.z + (rp.x + ay.x) / ang.tan();

    eprintln!("box_dims - {}", box_dims);
    eprintln!("gp - {}", gp);
    eprintln!("ay - {}", ay);
    eprintln!("ang - {}", ang);
    eprintln!("tan - {}", ang.tan());
    eprintln!("ax - {}", ax);
    eprintln!("az - {}", ax);

    vec3(ay.y, ax, az)
}

fn cast_voxel(rp: Vec3, rd: Vec3, vmin: Vec3, vmax: Vec3, scale: f32) -> bool {
    let mut tmin = 0.0;
    let mut tmax = 0.0;

    let found = ray_box_intersection(rp, rd, vmin, vmax, &mut tmin, &mut tmax);
    let g3d: Vec3 = (vmax - vmin) / (scale);

    if found {
        return false;
    }

    let rs = rp + tmin * rd;
    let re = rp + tmax * rd;
    let bsize = vmax - vmin;
    let vmax;
    let tmax;
    let vsize;
    let tdelta;

    let xi = Vec3::splat(1.0).max((rs - vmin / bsize).ceil());
    let exi = Vec3::splat(1.0).max((re - vmin / bsize).ceil());

    let mut p = vec3(
        ((((rs.x - vmin.x) / bsize.x) * g3d.x) + 1.).floor(),
        ((((rs.y - vmin.y) / bsize.y) * g3d.y) + 1.).floor(),
        ((((rs.z - vmin.z) / bsize.z) * g3d.z) + 1.).floor(),
    );

    if p.x == g3d.x + 1. {
        p.x = p.x - 1.;
    }

    if p.y == g3d.y + 1. {
        p.y = p.y - 1.;
    }

    if p.z == g3d.z + 1. {
        p.z = p.z - 1.;
    }

    let step = mask_neg(rd);
    let tvoxel = (p - (-1. * step)) / g3d;

    vmax = vmin + tvoxel * bsize;
    tmax = tmin + (vmax - rs) / rd;
    vsize = bsize / g3d;
    tdelta = vsize / rd.abs();

    eprintln!("tmin: {}", tmin);
    eprintln!("p: {}", p);
    eprintln!("g3d: {}", g3d);
    eprintln!("r: {}", rs);
    eprintln!("bsize: {}", bsize);
    eprintln!("tvoxel: {}", tvoxel);
    eprintln!("step: {}", step);
    eprintln!("vmax: {}", vmax);
    eprintln!("tmax: {}", tmax);
    eprintln!("vsize: {}", vsize);
    eprintln!("tdelta: {}", tdelta);

    true
}

fn select<T>(l: T, r: T, cond: bool) -> T {
    if cond {
        l
    } else {
        r
    }
}

fn ray_box_intersection(
    rp: Vec3,
    rd: Vec3,
    vmin: Vec3,
    vmax: Vec3,
    tmin: &mut f32,
    tmax: &mut f32,
) -> bool {
    let vmint = vec3(
        select(vmin.x, vmax.x, rd.x >= 0.),
        select(vmin.y, vmax.y, rd.y >= 0.),
        select(vmin.z, vmax.z, rd.z >= 0.),
    );
    let vmaxt = vec3(
        select(vmax.x, vmin.x, rd.x >= 0.),
        select(vmax.y, vmin.y, rd.y >= 0.),
        select(vmax.z, vmin.z, rd.z >= 0.),
    );

    let tmins = (vmint - rp) / rd;
    let tmaxs = (vmaxt - rp) / rd;

    if tmins.x > tmaxs.y || tmins.y > tmaxs.x {
        return false;
    }

    *tmin = tmins.x;
    *tmax = tmaxs.x;

    if tmins.y > *tmin {
        *tmin = tmins.y;
    }

    if tmaxs.y < *tmax {
        *tmax = tmaxs.y;
    }

    if *tmin > tmaxs.z || tmins.z > *tmax {
        return false;
    }

    if tmins.z > *tmin {
        *tmin = tmins.z;
    }

    if tmaxs.z < *tmax {
        *tmax = tmaxs.z;
    }

    return true;
}

#[cfg(test)]
mod tests {
    use crate::{
        shaders::compute::{calculate_full_depth, calculate_max_voxel},
        testing::octree::{MAX_BOUND, MIN_BOUND},
    };

    use super::*;
    use std::collections::HashSet;

    fn test_dim(dim: u32) {
        let parent_depth = dim;
        let mut positions = HashSet::new();

        for parent_x in 0..1 << parent_depth {
            for parent_y in 0..1 << parent_depth {
                for parent_z in 0..1 << parent_depth {
                    let parent_pos = UVec3::new(parent_x, parent_y, parent_z);
                    for child_x in 0..2 {
                        for child_y in 0..2 {
                            for child_z in 0..2 {
                                let child_rel_pos = UVec3::new(child_x, child_y, child_z);
                                let child_pos = get_child_pos(&parent_pos, &child_rel_pos);

                                assert!(child_pos.x < (1 << (parent_depth + 1)));
                                assert!(child_pos.y < (1 << (parent_depth + 1)));
                                assert!(child_pos.z < (1 << (parent_depth + 1)));
                                assert!(positions.insert(child_pos));
                            }
                        }
                    }
                }
            }
        }

        assert_eq!(
            positions.len(),
            ((1 << (parent_depth + 1)) as f32).powf(3.0) as usize
        );
    }

    fn test_dim_index(dim: u32) {
        let parent_depth = dim;
        let mut indices = HashSet::new();

        for parent_x in 0..1 << parent_depth {
            for parent_y in 0..1 << parent_depth {
                for parent_z in 0..1 << parent_depth {
                    let parent_pos = UVec3::new(parent_x, parent_y, parent_z);
                    for child_x in 0..2 {
                        for child_y in 0..2 {
                            for child_z in 0..2 {
                                let child_rel_pos = UVec3::new(child_x, child_y, child_z);
                                let child_pos = get_child_pos(&parent_pos, &child_rel_pos);

                                assert!(child_pos.x < (1 << (parent_depth + 1)));
                                assert!(child_pos.y < (1 << (parent_depth + 1)));
                                assert!(child_pos.z < (1 << (parent_depth + 1)));

                                let index = get_unique_index(&child_pos, parent_depth + 1);

                                println!("index: {}, pos: {:?}", index, child_pos);

                                assert!(indices.insert(index));
                            }
                        }
                    }
                }
            }
        }

        assert_eq!(
            indices.len(),
            ((1 << (parent_depth + 1)) as f32).powf(3.0) as usize
        );
    }

    fn test_dim_index_rec(i: u32) {
        let mut indices = HashSet::new();

        for dim in 0..i + 1 {
            let parent_depth = dim;

            for parent_x in 0..1 << parent_depth {
                for parent_y in 0..1 << parent_depth {
                    for parent_z in 0..1 << parent_depth {
                        let parent_pos = UVec3::new(parent_x, parent_y, parent_z);

                        assert!(parent_pos.x < (1 << (parent_depth)));
                        assert!(parent_pos.y < (1 << (parent_depth)));
                        assert!(parent_pos.z < (1 << (parent_depth)));

                        let id_bel = count_octrees_below(parent_depth, i);
                        let index = get_unique_index(&parent_pos, parent_depth);

                        println!(
                            "depth: {}, index: {}, pos: {:?}, count: {:?}",
                            dim, index, parent_pos, id_bel
                        );

                        assert!(indices.insert(id_bel + index));
                    }
                }
            }
        }

        assert_eq!(indices.len(), calculate_full_depth(i) as usize);
    }

    fn test_pos(dim: u32) {
        let parent_depth = dim;
        let mut positions = vec![];

        for parent_x in 0..1 << parent_depth {
            for parent_y in 0..1 << parent_depth {
                for parent_z in 0..1 << parent_depth {
                    let parent_pos = UVec3::new(parent_x, parent_y, parent_z);
                    for child_x in 0..2 {
                        for child_y in 0..2 {
                            for child_z in 0..2 {
                                let child_rel_pos = UVec3::new(child_x, child_y, child_z);
                                let child_pos = get_child_pos(&parent_pos, &child_rel_pos);

                                let pos = get_pos_from_grid_pos(
                                    &child_pos,
                                    parent_depth + 1,
                                    SETTINGS.scale,
                                );

                                let max_width: f32 =
                                    SETTINGS.scale / (1 << parent_depth + 1) as f32;
                                let min_scale =
                                    (SETTINGS.scale as f32 / 2.0) as f32 - (max_width / 2.0);

                                println!("pos: {:?}, cpos: {:?}", pos, child_pos);
                                println!("max_width: {}, scalediv2: {}", max_width, min_scale);

                                assert!(pos.x < min_scale);
                                assert!(pos.y < min_scale);
                                assert!(pos.z < min_scale);
                                assert!(pos.x > -min_scale);
                                assert!(pos.y > -min_scale);
                                assert!(pos.z > -min_scale);

                                positions.push(pos);
                            }
                        }
                    }
                }
            }
        }

        assert_eq!(
            positions.len(),
            ((1 << (parent_depth + 1)) as f32).powf(3.0) as usize
        );
    }

    fn assert_eq_print(i: u32, x: u32) {
        println!("actual: {}, expected: {}", i, x);
        assert!(i == x);
    }

    #[test]
    fn test_uniqueness_at_depth_1() {
        test_dim(1);
        test_dim_index(1);
        test_pos(1);
        test_dim_index_rec(1);
    }

    #[test]
    fn test_uniqueness_at_depth_2() {
        test_dim(2);
        test_dim_index(2);
        test_pos(2);
        test_dim_index_rec(2);
    }

    #[test]
    fn test_uniqueness_at_depth_3() {
        test_dim(3);
        test_dim_index(3);
        test_pos(3);
        test_dim_index_rec(3);
    }

    #[test]
    fn test_uniqueness_at_depth_4() {
        test_dim(4);
        test_dim_index(4);
        test_pos(4);
        test_dim_index_rec(4);
    }

    #[test]
    fn test_dimension_indexes() {
        assert_eq_print(count_octrees_below(2, 2), 0);
        assert_eq_print(count_octrees_below(1, 2), 64);
        assert_eq_print(count_octrees_below(0, 2), 72);

        assert_eq_print(count_octrees_below(3, 3), 0);
        assert_eq_print(count_octrees_below(2, 3), 512);
        assert_eq_print(count_octrees_below(1, 3), 576);
        assert_eq_print(count_octrees_below(0, 3), 584);
    }

    #[test]
    fn test_voxel() {
        assert_eq_print(calculate_max_voxel(1), 64);
        assert_eq_print(calculate_max_voxel(2), 512);
    }

    fn test_closest(dim: usize) {
        let lower = -(SETTINGS.scale / 2.0);
        let upper = SETTINGS.scale / 2.0;
        let delta = 0.01; // A small delta to test near-boundary conditions

        let half_dim = (dim as i32 / 2) as f32;

        assert_eq!(
            get_enclosed_octree(
                Vec3::new(lower + delta, lower + delta, lower + delta),
                dim,
                SETTINGS.scale
            ),
            IVec3::new(0, 0, 0)
        );

        assert_eq!(
            get_enclosed_octree(Vec3::new(0.0, 0.0, 0.0), dim, SETTINGS.scale),
            IVec3::new(half_dim as i32, half_dim as i32, half_dim as i32)
        );

        assert_eq!(
            get_enclosed_octree(
                Vec3::new(upper - delta, upper - delta, upper - delta),
                dim,
                SETTINGS.scale
            ),
            IVec3::new((dim - 1) as i32, (dim - 1) as i32, (dim - 1) as i32)
        );
    }

    #[test]
    fn closest() {
        test_closest(8);
        test_closest(4);
        test_closest(2);
        test_closest(1);

        let dim = 1;

        let bound = (SETTINGS.scale / 2.0) - 0.0001;

        assert_eq!(
            get_enclosed_octree(Vec3::new(-bound, -bound, -bound), dim, SETTINGS.scale),
            IVec3::new(0, 0, 0)
        );

        assert_eq!(
            get_enclosed_octree(Vec3::new(0.0, 0.0, 0.0), dim, SETTINGS.scale),
            IVec3::new(0, 0, 0)
        );

        assert_eq!(
            get_enclosed_octree(Vec3::new(bound, bound, bound), dim, SETTINGS.scale),
            IVec3::new(0, 0, 0)
        );
    }

    #[test]
    fn masking() {
        assert_eq!(mask_neg(Vec3::splat(-1.)), Vec3::splat(1.));
        assert_eq!(mask_neg(Vec3::splat(0.)), Vec3::splat(0.));
        assert_eq!(mask_neg(Vec3::splat(1.)), Vec3::splat(0.));

        assert_eq!(mask_pos(Vec3::splat(-1.)), Vec3::splat(0.));
        assert_eq!(mask_pos(Vec3::splat(0.)), Vec3::splat(0.));
        assert_eq!(mask_pos(Vec3::splat(1.)), Vec3::splat(1.));
    }

    #[test]
    fn next_grid_tests() {
        assert_eq!(
            cast_voxel(
                vec3(3.1, 3.2, 3.0),
                vec3(-1.0, -1.0, -1.0).normalize(),
                MIN_BOUND,
                MAX_BOUND,
                SETTINGS.scale
            ),
            false
        )
    }
}
