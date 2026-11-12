use std::{
    io::{self, BufRead},
    path::Path,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]

pub(crate) struct Tup3<T> {
    inner: (T, T, T),
}

impl<T> Tup3<T> {
    pub fn tup(&self) -> &(T, T, T) {
        &self.inner
    }
}

impl<T> From<Tup3<T>> for Vec<T> {
    fn from(value: Tup3<T>) -> Self {
        return vec![value.inner.0, value.inner.1, value.inner.2];
    }
}

impl<T> From<Tup3<T>> for [T; 3] {
    fn from(value: Tup3<T>) -> Self {
        return [value.inner.0, value.inner.1, value.inner.2];
    }
}

pub struct Model {
    verts: Vec<Tup3<f32>>,
    faces: Vec<Tup3<usize>>,
}

#[derive(Debug)]
pub enum Error {
    FileDoesNotExist,
}

impl Model {
    pub fn read<P: AsRef<Path>>(p: P) -> Result<Self, Error> {
        let file = std::fs::File::open(p).map_err(|_| Error::FileDoesNotExist)?;
        let reader = io::BufReader::new(file);

        let mut faces = vec![];
        let mut verts = vec![];

        for line in reader.lines().map_while(|l| l.ok()) {
            if line.starts_with("v ") {
                let subslice = &line[2..];

                let split: Vec<_> = subslice
                    .splitn(3, " ")
                    .map(str::parse::<f32>)
                    .map(Result::unwrap)
                    .collect();

                let tup = Tup3 {
                    inner: (split[0], split[1], split[2]),
                };
                verts.push(tup);
            } else if line.starts_with("f ") {
                // 'f 0123/123/3 124/5/76 5/433/21'

                let subslice = &line[2..];
                let split: Vec<_> = subslice
                    .splitn(3, " ")
                    .map(|piece| piece.split('/').next().unwrap().parse::<usize>())
                    .map(Result::unwrap)
                    .map(|x| x - 1)
                    .collect();

                faces.push(Tup3 {
                    inner: (split[0], split[1], split[2]),
                })
            }
        }

        println!("Faces: {}, Vertices: {}", faces.len(), verts.len());
        Result::Ok(Self { faces, verts })
    }

    pub fn vertices(&self) -> &Vec<Tup3<f32>> {
        &self.verts
    }

    pub fn faces(&self) -> &Vec<Tup3<usize>> {
        &self.faces
    }
}
