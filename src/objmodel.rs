use std::{
    io::{self, BufRead},
    ops::Index,
    path::Path,
};

use ndarray::Array3;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]

pub(crate) struct Tup3<T>(T, T, T);

impl<T> From<Tup3<T>> for Vec<T> {
    fn from(value: Tup3<T>) -> Self {
        vec![value.0, value.1, value.2]
    }
}

impl<T> From<Tup3<T>> for [T; 3] {
    fn from(value: Tup3<T>) -> Self {
        [value.0, value.1, value.2]
    }
}

impl<T> Index<usize> for Tup3<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.0,
            1 => &self.1,
            2 => &self.2,
            _ => panic!("invalid index {} for Tup3", index),
        }
    }
}

pub struct Model {
    verts: Vec<Tup3<f32>>,
    normals: Vec<Tup3<f32>>,
    uvw: Vec<Tup3<f32>>,

    faces: Vec<Tup3<usize>>,        // indices into `verts`
    face_normals: Vec<Tup3<usize>>, // indices into `normals`
    face_uvs: Vec<Tup3<usize>>,     // indices into uvw

    normals_texture: Option<Array3<u8>>,
    diffuse_texture: Option<Array3<u8>>,
    specular_texture: Option<Array3<u8>>,
}

#[derive(Debug)]
pub enum Error {
    FileDoesNotExist,
}

struct FData {
    pub vert: usize,
    pub uv: usize,
    pub norm: usize,
}

impl Model {
    fn parse_float_triple(subslice: &str) -> Tup3<f32> {
        let split: Vec<_> = subslice
            .split_ascii_whitespace()
            .map(str::parse::<f32>)
            .map(Result::unwrap)
            .collect();

        Tup3(split[0], split[1], split[2])
    }

    fn parse_v_line(line: &str) -> Tup3<f32> {
        let subslice = &line[2..];
        Model::parse_float_triple(subslice)
    }

    fn parse_vt_line(line: &str) -> Tup3<f32> {
        let subslice = &line[3..];
        Model::parse_float_triple(subslice)
    }

    fn parse_vn_line(line: &str) -> Tup3<f32> {
        let subslice = &line[3..];
        Model::parse_float_triple(subslice)
    }

    fn parse_f_line(line: &str) -> Vec<FData> {
        // 'f 0123/123/3 124/5/76 5/433/21'

        let subslice = &line[2..];
        subslice
            .split_ascii_whitespace()
            .map(|piece| {
                let pieces = piece.split('/');
                let [vert_ix, uv_ix, norm_ix] = pieces.collect::<Vec<_>>()[..] else {
                    panic!("couldn't parse obj file");
                };
                FData {
                    vert: vert_ix.parse::<usize>().unwrap() - 1,
                    uv: uv_ix.parse::<usize>().unwrap() - 1,
                    norm: norm_ix.parse::<usize>().unwrap() - 1,
                }
            })
            .collect()
    }

    pub fn read<P>(p: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        let file = std::fs::File::open(p).map_err(|_| Error::FileDoesNotExist)?;
        let reader = io::BufReader::new(file);

        let mut verts = vec![];
        let mut normals = vec![];
        let mut uvs = vec![];
        let mut faces = vec![];
        let mut face_normals = vec![];
        let mut face_uvs = vec![];

        for line in reader.lines().map_while(Result::ok) {
            if line.starts_with("v ") {
                verts.push(Model::parse_v_line(&line));
            } else if line.starts_with("vn ") {
                normals.push(Model::parse_vn_line(&line));
            } else if line.starts_with("vt ") {
                uvs.push(Model::parse_vt_line(&line));
            } else if line.starts_with("f ") {
                let split = Model::parse_f_line(&line);
                faces.push(Tup3(split[0].vert, split[1].vert, split[2].vert));
                face_uvs.push(Tup3(split[0].uv, split[1].uv, split[2].uv));
                face_normals.push(Tup3(split[0].norm, split[1].norm, split[2].norm));
            }
        }

        println!("Faces: {}, Vertices: {}", faces.len(), verts.len());
        Ok(Self {
            faces,
            verts,
            uvw: uvs,
            normals,
            face_normals,
            face_uvs,
            normals_texture: None,
            diffuse_texture: None,
            specular_texture: None,
        })
    }

    pub fn add_normal_texture(&mut self, tex: Array3<u8>) {
        self.normals_texture = Some(tex);
    }

    pub fn add_diffuse_texture(&mut self, tex: Array3<u8>) {
        self.diffuse_texture = Some(tex)
    }

    pub fn add_specular_texture(&mut self, tex: Array3<u8>) {
        self.specular_texture = Some(tex)
    }

    pub fn vertices(&self) -> &Vec<Tup3<f32>> {
        &self.verts
    }

    pub fn faces(&self) -> &Vec<Tup3<usize>> {
        &self.faces
    }

    pub fn uvs(&self) -> &Vec<Tup3<f32>> {
        &self.uvw
    }

    pub fn normals(&self) -> &Vec<Tup3<f32>> {
        &self.normals
    }

    pub fn face_normals(&self) -> &Vec<Tup3<usize>> {
        &self.face_normals
    }

    pub fn face_uvs(&self) -> &Vec<Tup3<usize>> {
        &self.face_uvs
    }

    pub fn normals_texture(&self) -> Option<&Array3<u8>> {
        self.normals_texture.as_ref()
    }

    pub fn specular_texture(&self) -> Option<&Array3<u8>> {
        self.specular_texture.as_ref()
    }

    pub fn diffuse_texture(&self) -> Option<&Array3<u8>> {
        self.diffuse_texture.as_ref()
    }
}
