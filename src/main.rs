use std::{
    env::args,
    path::{Path, PathBuf},
    process::ExitCode,
};

use clap::Parser;
use ndarray::prelude::*;

mod camera;
mod geom;
mod linalg;
mod objmodel;
mod shader;
mod viewport;
use objmodel::Model;
use serde::{Deserialize, Serialize};
use zenbitmaps::DecodeOutput;

use crate::{camera::Camera, shader::phong::PhongShader};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Arguments {
    model_file: Option<PathBuf>,
    #[arg(short, long)]
    normal_map_file: Option<PathBuf>,
    #[arg(short, long)]
    diff_texture_file: Option<PathBuf>,
    #[arg(short, long)]
    spec_texture_file: Option<PathBuf>,

    #[arg(short, long)]
    config: Option<PathBuf>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ModelSpec {
    obj_file: PathBuf,
    normal_map_file: Option<PathBuf>,
    diff_texture_file: Option<PathBuf>,
    spec_texture_file: Option<PathBuf>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SceneConfig {
    model: Vec<ModelSpec>,
}

fn usage() -> &'static str {
    "Usage: ./tinyrenderer-rs <filename.obj> [normals.tga]"
}

const WIDTH: u32 = 2048;
const HEIGHT: u32 = 2048;

fn draw<I>(
    models: I,
    width: u32,
    height: u32,
    eye: &Array1<f32>,
    center: &Array1<f32>,
    up: &Array1<f32>,
) -> (Array3<u8>, Array2<f32>)
where
    I: IntoIterator<Item = Model>,
{
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
    for model in models {
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
        zenbitmaps::PixelLayout::Gray8 => {
            let pixels = tga_data.pixels();
            let array = Array::from_vec(pixels.into());
            array
                .to_shape((height as usize, width as usize, 1))
                .unwrap()
                .to_owned()
        }
        _ => unreachable!(),
    }
}

fn read_tga_to_ndarray<P>(path: P) -> Array3<u8>
where
    P: AsRef<Path>,
{
    let data = std::fs::read(path).expect("failed to open provided TGA file");
    let tga_data = zenbitmaps::decode_tga(&data, zenbitmaps::Unstoppable)
        .expect("failed to decode .tga file")
        .into_owned();
    to_ndarray_format(tga_data)
}

fn load_model_data<P>(
    model_fname: P,
    normal_map: Option<P>,
    spec_map: Option<P>,
    diff_map: Option<P>,
) -> Model
where
    P: AsRef<Path>,
{
    let mut model = Model::read(model_fname).expect("failed to read .obj");

    let normals = normal_map.map(read_tga_to_ndarray);
    let spec_tex = spec_map.map(read_tga_to_ndarray);
    let diff_tex = diff_map.map(read_tga_to_ndarray);
    if let Some(normal_data) = normals {
        model.add_normal_texture(normal_data)
    }
    if let Some(specular_data) = spec_tex {
        model.add_specular_texture(specular_data);
    }
    if let Some(diffuse_data) = diff_tex {
        model.add_diffuse_texture(diffuse_data);
    }
    model
}

fn draw_from_config_file(
    config_file: PathBuf,
    eye: &Array1<f32>,
    center: &Array1<f32>,
    up: &Array1<f32>,
) -> (Array3<u8>, Array2<f32>) {
    let config: SceneConfig = toml::from_slice(&std::fs::read(config_file).unwrap()).unwrap();
    let models = config.model.into_iter().map(|model_spec| {
        load_model_data(
            model_spec.obj_file,
            model_spec.normal_map_file,
            model_spec.spec_texture_file,
            model_spec.diff_texture_file,
        )
    });
    draw(models, WIDTH, HEIGHT, &eye, &center, &up)
}

fn main() -> ExitCode {
    let arguments = Arguments::parse();

    let eye: Array1<f32> = array![-1.0, 0.0, 2.0];
    let center: Array1<f32> = array![0.0, 0.0, 0.0];
    let up: Array1<f32> = array![0.0, 1.0, 0.0];

    let (fbuffer, dbuffer) = if let Some(config_file) = arguments.config {
        draw_from_config_file(config_file, &eye, &center, &up)
    } else {
        let model_filename = arguments.model_file.unwrap();
        let model = [load_model_data(
            model_filename,
            arguments.normal_map_file,
            arguments.spec_texture_file,
            arguments.diff_texture_file,
        )];

        draw(model, WIDTH, HEIGHT, &eye, &center, &up)
    };

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
