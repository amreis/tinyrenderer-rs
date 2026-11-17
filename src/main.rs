use std::{env::args, process::ExitCode};

use ndarray::{prelude::*, stack};

mod objmodel;
use objmodel::Model;

fn usage() -> &'static str {
    "Usage: ./tinyrenderer-rs <filename.obj>"
}

const WIDTH: u32 = 2048;
const HEIGHT: u32 = 2048;

fn triangle(
    mut a: Array1<f32>,
    mut b: Array1<f32>,
    mut c: Array1<f32>,
    viewport_t: &Array2<f32>,
    framebuffer: &mut Array3<u8>,
    depthbuffer: &mut Array2<f32>,
    color: &[u8; 3],
) {
    a /= a[3];
    b /= b[3];
    c /= c[3];

    let a_i: ArrayBase<ndarray::OwnedRepr<i32>, Dim<[usize; 1]>, i32> =
        viewport_t.dot(&a).round().mapv(|x| x as i32);
    let b_i = viewport_t.dot(&b).round().mapv(|x| x as i32);
    let c_i = viewport_t.dot(&c).round().mapv(|x| x as i32);

    let m = stack![Axis(0), a_i, b_i, c_i];
    let &xmin = m.column(0).iter().min().unwrap();
    let &xmax = m.column(0).iter().max().unwrap();
    let &ymin = m.column(1).iter().min().unwrap();
    let &ymax = m.column(1).iter().max().unwrap();

    let ab_x = b_i[0] - a_i[0];
    let ab_y = b_i[1] - a_i[1];
    let bc_x = c_i[0] - b_i[0];
    let bc_y = c_i[1] - b_i[1];

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
            let area_opp_a = (y_ - b_i[1]) * (c_i[0] - b_i[0]) - (x_ - b_i[0]) * (c_i[1] - b_i[1]);
            let area_opp_b = (y_ - c_i[1]) * (a_i[0] - c_i[0]) - (x_ - c_i[0]) * (a_i[1] - c_i[1]);
            let area_opp_c = total_area - area_opp_a - area_opp_b;

            let has_neg = (area_opp_a < 0) || (area_opp_b < 0) || (area_opp_c < 0);
            if has_neg {
                continue;
            }

            let alpha = area_opp_a as f32 / total_area as f32;
            let beta = area_opp_b as f32 / total_area as f32;
            let gamma = area_opp_c as f32 / total_area as f32;
            let depth = alpha * a[2] as f32 + beta * b[2] as f32 + gamma * c[2] as f32;
            if let Some(idepth) = depthbuffer.get_mut([HEIGHT as usize - y - 1, x])
                && depth >= *idepth
            {
                let mut slice = framebuffer.slice_mut(s![HEIGHT as usize - y - 1, x, ..]);
                slice.assign(&arr1(color));
                *idepth = depth;
            }
        }
    }
}

fn viewport(x: i32, y: i32, w: u32, h: u32) -> Array2<f32> {
    array![
        [w as f32 / 2.0, 0., 0., x as f32 + w as f32 / 2.0],
        [0., h as f32 / 2.0, 0., y as f32 + h as f32 / 2.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

fn perspmat(f: f32) -> Array2<f32> {
    array![
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, -1. / f, 1.0]
    ]
}

fn cross_prod(u: &Array1<f32>, v: &Array1<f32>) -> Array1<f32> {
    debug_assert!(u.len() == v.len() && u.len() == 3);

    array![
        u[1] * v[2] - u[2] * v[1],
        u[2] * v[0] - u[0] * v[2],
        u[0] * v[1] - u[1] * v[0]
    ]
}

fn normalize(v: &mut Array1<f32>) {
    *v /= v.mapv(|x| x * x).sum().sqrt();
}

fn camera(eye: &Array1<f32>, center: &Array1<f32>, up: &Array1<f32>) -> Array2<f32> {
    let mut n = eye - center;
    normalize(&mut n);

    let mut l = cross_prod(&up, &n);
    normalize(&mut l);

    let mut m = cross_prod(&n, &l);
    normalize(&mut m);

    let submat = stack![Axis(0), l, m, n];

    let mut firstblock = Array2::<f32>::eye(4);
    firstblock.slice_mut(s![0..3, 0..3]).assign(&submat);

    let mut translateblock = Array2::<f32>::eye(4);
    translateblock.slice_mut(s![0..3, 3]).assign(&-center);

    firstblock.dot(&translateblock)
}

fn draw(
    model: Model,
    width: u32,
    height: u32,
    eye: &Array1<f32>,
    center: &Array1<f32>,
    up: &Array1<f32>,
) -> (Array3<u8>, Array2<f32>) {
    let mut framebuffer = Array3::zeros((height as usize, width as usize, 3));
    let mut depthbuffer = Array2::from_elem((height as usize, width as usize), -5.0);
    let model_view_t = camera(eye, center, up);
    let perspective_t = perspmat((eye - center).mapv(|x| x * x).sum().sqrt());
    let viewport_t = viewport(
        WIDTH as i32 / 16,
        HEIGHT as i32 / 16,
        (WIDTH * 7) / 8,
        (HEIGHT * 7) / 8,
    );

    let persp_and_mv = perspective_t.dot(&model_view_t);

    for &face in model.faces() {
        let [f1, f2, f3] = Into::<[usize; 3]>::into(*face.tup())
            .map(|x| model.vertices()[x].into())
            .map(Array1::from_vec)
            .map(|a| persp_and_mv.dot(&array![a[0], a[1], a[2], 1.0]));
        let color: [u8; 3] = rand::random();

        triangle(
            f1,
            f2,
            f3,
            &viewport_t,
            &mut framebuffer,
            &mut depthbuffer,
            &color,
        )
    }

    (framebuffer, depthbuffer)
}

fn main() -> ExitCode {
    let args = args();
    let filename = match args.skip(1).next() {
        Some(val) => val,
        None => {
            println!("{}", usage());
            return ExitCode::FAILURE;
        }
    };

    let eye: Array1<f32> = array![-1.0, 0.0, 2.0];
    let center: Array1<f32> = array![0.0, 0.0, 0.0];
    let up: Array1<f32> = array![0.0, 1.0, 0.0];

    let model = Model::read(filename).expect("failed to read .obj");

    let (fbuffer, dbuffer) = draw(model, WIDTH, HEIGHT, &eye, &center, &up);
    let pixels = fbuffer.flatten().to_vec();

    match zenbitmaps::encode_tga(
        &pixels,
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
        Err(err) => {
            std::println!("{:?}", err);
            return ExitCode::FAILURE;
        }
    }

    ExitCode::SUCCESS
}
