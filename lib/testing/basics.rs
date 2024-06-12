// NOTE this is not really rust like code, I'm writing it as if i'm writing it in wgsl, so no options, errors or tuples.

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
    let mut tmin: f32 = 0.0;
    let mut tmax = 0.0;

    let found = ray_box_intersection(rp, rd, vmin, vmax, &mut tmin, &mut tmax);
    eprintln!("tmin: {}", tmin);
    eprintln!("tmax: {}", tmax);

    if !found {
        return false;
    }

    let g3d: Vec3 = (vmax - vmin) / (scale);

    let ray_start = rp + rd * tmin;
    let ray_end = rp + rd * tmax;
    let bsize = scale;

    let mut current_index = Vec3::splat(1.0).max(((ray_start - vmin) / bsize).ceil());
    let end_index = Vec3::splat(1.0).max((ray_end - vmin / bsize).ceil());

    let step = sign(rd);
    let tdelta = bsize / (rd.abs());
    let mut tmax = tmin + (vmin + (current_index * step) * bsize - ray_start / rd);

    eprintln!("g3d: {}", g3d);
    eprintln!("rd: {}", rd);
    eprintln!("r: {}", ray_start);
    eprintln!("re: {}", ray_end);
    eprintln!("bsize: {}", bsize);
    eprintln!("step: {}", step);
    eprintln!("vmax: {}", vmax);
    eprintln!("tmax: {}", tmax);
    eprintln!("tdelta: {}", tdelta);
    eprintln!("xi: {}", current_index);
    eprintln!("exi: {}", end_index);

    let max_iter = 100;
    let mut i = 0;

    let mut dmut = tdelta;

    while (current_index.x != end_index.x
        || current_index.y != end_index.y
        || current_index.z != end_index.z)
        && i < max_iter
    {
        eprintln!("tmax: {}", tmax);
        eprintln!("Intersection: voxel = {}", current_index);
        eprintln!("Intersection: pos = {}", rp + (rd * tdelta * (i + 1) as f32));

        i += 1;

        if tmax.x < tmax.y && tmax.x < tmax.z {
            // x-axis traversal.
            current_index.x += step.x;
            tmax.x += tdelta.x;
        } else if tmax.y < tmax.z {
            // y-axis traversal.
            current_index.y += step.y;
            tmax.y += tdelta.y;
        } else {
            // z-axis traversal.
            current_index.z += step.z;
            tmax.z += tdelta.z;
        }
    }

    eprintln!("tmax: {}", tmax);
    eprintln!("Intersection: voxel = {}", current_index);
    eprintln!("Intersection: pos = {}", rp + (rd * dmut));

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
    let rdi = 1. / rd;

    if rdi.x >= 0. {
        *tmin = (vmin.x - rp.x) * rdi.x;
        *tmax = (vmax.x - rp.x) * rdi.x;
    } else {
        *tmin = (vmax.x - rp.x) * rdi.x;
        *tmax = (vmin.x - rp.x) * rdi.x;
    }

    let tymin;
    let tymax;
    if rdi.y >= 0. {
        tymin = (vmin.y - rp.y) * rdi.y;
        tymax = (vmax.y - rp.y) * rdi.y;
    } else {
        tymin = (vmax.y - rp.y) * rdi.y;
        tymax = (vmin.y - rp.y) * rdi.y;
    }

    if *tmin > tymax || tymin > *tmax {
        return false;
    }
    if tymin > *tmin {
        *tmin = tymin;
    };
    if tymax < *tmax {
        *tmax = tymax;
    }

    let tzmin;
    let tzmax;
    if rdi.z >= 0. {
        tzmin = (vmin.z - rp.z) * rdi.z;
        tzmax = (vmax.z - rp.z) * rdi.z;
    } else {
        tzmin = (vmax.z - rp.z) * rdi.z;
        tzmax = (vmin.z - rp.z) * rdi.z;
    }

    if *tmin > tzmax || tzmin > *tmax {
        return false;
    }
    if tzmin > *tmin {
        *tmin = tzmin;
    }
    if tzmax < *tmax {
        *tmax = tzmax;
    }

    true
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
                vec3(3., 3., 3.0),
                vec3(-1.0, -1.0, -1.0).normalize(),
                MIN_BOUND,
                MAX_BOUND,
                SETTINGS.scale / 2.0
            ),
            false
        )
    }
}
