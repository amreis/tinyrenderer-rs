use std::{env::args, process::ExitCode};

use ndarray::{prelude::*, stack};

mod camera;
mod geom;
mod linalg;
mod objmodel;
mod shader;
mod viewport;
use objmodel::Model;

use crate::{
    camera::Camera,
    shader::{phong::PhongShader, random::RandomShader, shader::Color},
};

fn usage() -> &'static str {
    "Usage: ./tinyrenderer-rs <filename.obj>"
}

const WIDTH: u32 = 2048;
const HEIGHT: u32 = 2048;

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
    let camera = Camera::new()
        .look_at(eye, center, up)
        .perspective(linalg::norm(&(eye - center)));
    let viewport_t = viewport::Viewport::new(
        WIDTH as i32 / 16,
        HEIGHT as i32 / 16,
        (WIDTH * 7) / 8,
        (HEIGHT * 7) / 8,
    );
    let mut shader = PhongShader::new(&model, arr1(&[0.0, 0.0, 1.0]), &camera);

    for i in 0..model.faces().len() {
        shader.set_color(rand::random());
        let t = geom::Triangle::new(
            shader.vertex(i, 0),
            shader.vertex(i, 1),
            shader.vertex(i, 2),
        );

        t.rasterize(&viewport_t, &shader, &mut framebuffer, &mut depthbuffer);
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
