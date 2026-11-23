use std::{
    io::{self, BufRead},
    path::Path,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]

pub(crate) struct Tup3<T> {
    inner: (T, T, T),
}

impl<T> From<Tup3<T>> for Vec<T> {
    fn from(value: Tup3<T>) -> Self {
        vec![value.inner.0, value.inner.1, value.inner.2]
    }
}

impl<T> From<Tup3<T>> for [T; 3] {
    fn from(value: Tup3<T>) -> Self {
        [value.inner.0, value.inner.1, value.inner.2]
    }
}

pub struct Model {
    verts: Vec<Tup3<f32>>,
    normals: Vec<Tup3<f32>>,
    uvw: Vec<Tup3<f32>>,

    faces: Vec<Tup3<usize>>,        // indices into `verts`
    face_normals: Vec<Tup3<usize>>, // indices into `normals`
    face_uvs: Vec<Tup3<usize>>,     // indices into uvw
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
    fn parse_v_line(line: &str) -> Tup3<f32> {
        let subslice = &line[2..];

        let split: Vec<_> = subslice
            .split_ascii_whitespace()
            .map(str::parse::<f32>)
            .map(Result::unwrap)
            .collect();

        Tup3 {
            inner: (split[0], split[1], split[2]),
        }
    }

    fn parse_vt_line(line: &str) -> Tup3<f32> {
        let subslice = &line[2..];
        let split: Vec<_> = subslice
            .split_ascii_whitespace()
            .map(str::parse::<f32>)
            .map(Result::unwrap)
            .collect();

        Tup3 {
            inner: (split[0], split[1], split[2]),
        }
    }

    fn parse_vn_line(line: &str) -> Tup3<f32> {
        let subslice = &line[2..];
        let split: Vec<_> = subslice
            .split_ascii_whitespace()
            .map(str::parse::<f32>)
            .map(Result::unwrap)
            .collect();

        Tup3 {
            inner: (split[0], split[1], split[2]),
        }
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

    pub fn read<P: AsRef<Path>>(p: P) -> Result<Self, Error> {
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
                faces.push(Tup3 {
                    inner: (split[0].vert, split[1].vert, split[2].vert),
                });
                face_uvs.push(Tup3 {
                    inner: (split[0].uv, split[1].uv, split[2].uv),
                });
                face_normals.push(Tup3 {
                    inner: (split[0].norm, split[1].norm, split[2].norm),
                });
            }
        }

        println!("Faces: {}, Vertices: {}", faces.len(), verts.len());
        Result::Ok(Self {
            faces,
            verts,
            normals,
            face_normals,
        })
    }

    pub fn vertices(&self) -> &Vec<Tup3<f32>> {
        &self.verts
    }

    pub fn faces(&self) -> &Vec<Tup3<usize>> {
        &self.faces
    }

    pub fn normals(&self) -> &Vec<Tup3<f32>> {
        &self.normals
    }

    pub fn face_normals(&self) -> &Vec<Tup3<usize>> {
        &self.face_normals
    }
}
