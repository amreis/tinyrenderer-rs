use std::env::args;

use ndarray::prelude::*;

mod objmodel;
use objmodel::Model;

fn usage() -> &'static str {
    "Usage: ./tinyrenderer-rs <filename.obj>"
}

const WIDTH: u32 = 2048;
const HEIGHT: u32 = 2048;

fn rotate(point: Array1<f32>) -> Array1<f32> {
    const ANGLE: f32 = std::f32::consts::PI / 6.0;
    let mat: Array2<f32> = array![
        [ANGLE.cos(), 0.0, ANGLE.sin()],
        [0.0, 1.0, 0.0],
        [-ANGLE.sin(), 0.0, ANGLE.cos()]
    ];

    mat.dot(&point)
}

fn perspective(point: Array1<f32>) -> Array1<f32> {
    const C: f32 = 3.0;
    &point / (1.0 - &point[2] / C)
}

fn project(point: Array1<f32>, w: u32, h: u32) -> Array1<i32> {
    let interm = (point + 1.0) * &array![(w as f32) / 2., (h as f32) / 2., 256.0 / 2.0];

    interm.round().mapv(|x| x as i32)
}

fn triangle(
    ax: i32,
    ay: i32,
    az: i32,
    bx: i32,
    by: i32,
    bz: i32,
    cx: i32,
    cy: i32,
    cz: i32,
    framebuffer: &mut Array3<u8>,
    depthbuffer: &mut Array2<f32>,
    color: &[u8; 3],
) {
    let &xmin = [ax, bx, cx].iter().min().unwrap();
    let &ymin = [ay, by, cy].iter().min().unwrap();
    let &xmax = [ax, bx, cx].iter().max().unwrap();
    let &ymax = [ay, by, cy].iter().max().unwrap();

    let ab_x = bx - ax;
    let ab_y = by - ay;
    let bc_x = cx - bx;
    let bc_y = cy - by;

    let total_area = ab_x * bc_y - ab_y * bc_x;
    if total_area < 1 {
        return;
    }

    // let adjugate = array![[by - cy, cx - bx], [cy - ay, ax - cx]];

    let [lbx, ubx, lby, uby] = [
        xmin.max(0) as usize,
        (xmax as usize).min(framebuffer.len_of(Axis(1)) - 1),
        ymin.max(0) as usize,
        (ymax as usize).min(framebuffer.len_of(Axis(0)) - 1),
    ];
    for x in lbx..=ubx {
        for y in lby..=uby {
            let y_ = y as i32;
            let x_ = x as i32;
            let area_opp_a = (y_ - by) * (cx - bx) - (x_ - bx) * (cy - by);
            let area_opp_b = (y_ - cy) * (ax - cx) - (x_ - cx) * (ay - cy);
            let area_opp_c = total_area - area_opp_a - area_opp_b;

            let has_neg = (area_opp_a < 0) || (area_opp_b < 0) || (area_opp_c < 0);
            if has_neg {
                continue;
            }

            let alpha = area_opp_a as f32 / total_area as f32;
            let beta = area_opp_b as f32 / total_area as f32;
            let gamma = area_opp_c as f32 / total_area as f32;

            let depth = alpha * az as f32 + beta * bz as f32 + gamma * cz as f32;

            if let Some(idepth) = depthbuffer.get_mut([HEIGHT as usize - y - 1, x])
                && depth > *idepth
            {
                let mut slice = framebuffer.slice_mut(s![HEIGHT as usize - y - 1, x, ..]);
                slice.assign(&arr1(&color.clone()));
                *idepth = depth;
            }
        }
    }
}

fn draw(model: Model, width: u32, height: u32) -> (Array3<u8>, Array2<f32>) {
    let mut framebuffer = Array3::zeros((height as usize, width as usize, 3));
    let mut depthbuffer = Array2::from_elem((height as usize, width as usize), -256.0);

    for &face in model.faces() {
        let [f1, f2, f3] = Into::<[usize; 3]>::into(*face.tup())
            .map(|x| model.vertices()[x].into())
            .map(Array1::from_vec)
            .map(rotate)
            .map(perspective)
            .map(|p| project(p, width, height));

        let color = rand::random::<[u8; 3]>();

        triangle(
            f1[0],
            f1[1],
            f1[2],
            f2[0],
            f2[1],
            f2[2],
            f3[0],
            f3[1],
            f3[2],
            &mut framebuffer,
            &mut depthbuffer,
            &color,
        )
    }

    (framebuffer, depthbuffer)
}

fn main() {
    let args = args();
    let filename = match args.skip(1).next() {
        Some(val) => val,
        None => {
            println!("{}", usage());
            return;
        }
    };

    let model = Model::read(filename).expect("failed to read .obj");

    let (fbuffer, dbuffer) = draw(model, WIDTH, HEIGHT);
    let pixels = fbuffer.flatten();

    match zenbitmaps::encode_tga(
        &pixels.to_vec(),
        WIDTH,
        HEIGHT,
        zenbitmaps::PixelLayout::Rgb8,
        zenbitmaps::Unstoppable,
    ) {
        Ok(data) => std::fs::write("out.tga", data).expect("Failed to write file out.tga"),
        Err(err) => std::println!("{:?}", err),
    }

    let min_depth = dbuffer.iter().fold(f32::INFINITY, |a, &b| a.min(b));
    let max_depth = dbuffer.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));

    match zenbitmaps::encode_tga(
        &dbuffer
            .mapv(|x| (255.0 * (x - min_depth) / (max_depth - min_depth)).round() as u8)
            .flatten()
            .to_vec(),
        WIDTH,
        HEIGHT,
        zenbitmaps::PixelLayout::Gray8,
        zenbitmaps::Unstoppable,
    ) {
        Ok(data) => std::fs::write("dbuffer.tga", data).expect("Failed to write dbuffer.tga"),
        Err(err) => std::println!("{:?}", err),
    }
}
