use std::{env::args, process::ExitCode};

use ndarray::prelude::*;

mod camera;
mod geom;
mod linalg;
mod objmodel;
mod shader;
mod viewport;
use objmodel::Model;
use zenbitmaps::DecodeOutput;

use crate::{camera::Camera, shader::phong::PhongShader};

fn usage() -> &'static str {
    "Usage: ./tinyrenderer-rs <filename.obj> [normals.tga]"
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

fn to_ndarray_format(tga_data: DecodeOutput) -> Array3<u8> {
    let (height, width, layout) = (tga_data.height, tga_data.width, tga_data.layout);
    // layout will be Rgb8, Rgba8, or Gray8, by the documentation of decode_tga()
    let tga_data = tga_data.into_owned();
    match layout {
        zenbitmaps::PixelLayout::Rgb8 => {
            let pixels = tga_data.pixels();
            let array = Array::from_vec(pixels.into());
            array
                .to_shape((height as usize, width as usize, 3usize))
                .unwrap()
                .to_owned()
        }
        zenbitmaps::PixelLayout::Rgba8 => {
            let pixels = tga_data.pixels();
            let array = Array::from_vec(
                pixels
                    .iter()
                    .enumerate()
                    .filter_map(|(ix, &elt)| if ix % 4 == 3 { None } else { Some(elt) })
                    .collect::<Vec<_>>()
                    .into(),
            );
            array
                .to_shape((height as usize, width as usize, 3usize))
                .unwrap()
                .to_owned()
        }
        _ => unreachable!(),
    }
}

fn main() -> ExitCode {
    let mut args = args().skip(1); // skip over program name
    let Some(model_filename) = args.next() else {
        println!("{}", usage());
        return ExitCode::FAILURE;
    };
    let mut model = Model::read(model_filename).expect("failed to read .obj");

    let normals = args
        .next()
        .map(|normals_filename| {
            std::fs::read(&normals_filename).expect("failed to open provided normals file")
        })
        .map(|data| {
            zenbitmaps::decode_tga(&data, zenbitmaps::Unstoppable)
                .expect("failed to decode .tga file")
                .into_owned()
        })
        .inspect(|normal_data| {
            println!(
                "Normals file has height {} and width {} pixels and layout {:?}",
                normal_data.height, normal_data.width, normal_data.layout,
            );
        })
        .map(to_ndarray_format);

    let eye: Array1<f32> = array![-1.0, 0.0, 2.0];
    let center: Array1<f32> = array![0.0, 0.0, 0.0];
    let up: Array1<f32> = array![0.0, 1.0, 0.0];

    if let Some(normal_data) = normals {
        model.add_texture(normal_data)
    };
    let model = model;

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
