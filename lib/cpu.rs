use cgmath::{vec2, vec3, Array, InnerSpace, Vector2, Vector3};

use bevy_shade_lib::testing::basics::castf2;

const RESOLUTION: u32 = 240;
const MAX_DEPTH: u32 = 3;

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
fn closest_octree(rp: Vector3<f32>, rd: Vector3<f32>, dim: &mut u32) -> f32 {
    rp.dot(rp).sqrt() - 0.5
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

pub fn cast_ray(ro: Vector3<f32>, rd: Vector3<f32>) -> f32 {
    let mut t = 0.0;
    let mut depth = 0;

    for _ in 0..200 {
        let pos = ro + t * rd;

        let d = closest_octree(pos, rd, &mut depth);

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

fn calc_normal(pos: Vector3<f32>) -> Vector3<f32> {
    let e = vec2(0.0001, 0.0);
    let mut d = MAX_DEPTH;

    return (vec3(
        closest_octree(pos + vec3(e.x, e.y, e.y), vec3(0.0, 0.0, 0.0), &mut d)
            - closest_octree(pos - vec3(e.x, e.y, e.y), vec3(0.0, 0.0, 0.0), &mut d),
        closest_octree(pos + vec3(e.y, e.x, e.y), vec3(0.0, 0.0, 0.0), &mut d)
            - closest_octree(pos - vec3(e.y, e.x, e.y), vec3(0.0, 0.0, 0.0), &mut d),
        closest_octree(pos + vec3(e.y, e.y, e.x), vec3(0.0, 0.0, 0.0), &mut d)
            - closest_octree(pos - vec3(e.y, e.y, e.x), vec3(0.0, 0.0, 0.0), &mut d),
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
fn render(ro: Vector3<f32>, rd: Vector3<f32>) -> Vector3<f32> {
    let t = cast_ray(ro, rd);

    if t > 0. {
        let pos = ro + t * rd;
        return calc_normal(pos);
    }

    vec3(0.0, 0.0, 0.0)
}

fn fragment(pos: Vector2<u32>) -> Vector3<f32> {
    // custom uv, not quite the same as in.uv.
    let r = Vector2::new(RESOLUTION as f32, RESOLUTION as f32);
    let uv: Vector2<f32> = ((castf2(&pos) * 2.) - r) / r.y;

    // rotation around 0.,,
    let ro = vec3(0.0, 0.0, -2.);

    // todo convert this from linear algebra rotation to geometric algebra.
    let ta = vec3(0., 0., 0.);
    let ww = (ta - ro).normalize();
    let uu = (ww.cross(vec3(0.0, 1.0, 0.0))).normalize();
    let vv = (uu.cross(ww)).normalize();

    let rd: Vector3<f32> = (uv.x * uu + uv.y * vv + 2.0 * ww).normalize();

    render(ro, rd)
}

fn main() {
    println!("P3\n{} {}\n255", RESOLUTION, RESOLUTION);

    (0..RESOLUTION)
        .into_iter()
        .map(|x| {
            (0..RESOLUTION)
                .into_iter()
                .map(move |y| Vector2::new(x.clone(), y.clone()))
        })
        .flatten()
        .map(fragment)
        .for_each(|col| {
            let ir = (255.99 * col.x) as u8;
            let ig = (255.99 * col.y) as u8;
            let ib = (255.99 * col.z) as u8;

            println!("{} {} {}", ir, ig, ib);
        });
}
