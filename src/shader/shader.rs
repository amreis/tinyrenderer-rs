use ndarray::prelude::*;
use rand::{
    RngExt,
    distr::{Distribution, StandardUniform},
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
pub struct Color(u8, u8, u8);

impl Distribution<Color> for StandardUniform {
    fn sample<R: rand::prelude::Rng + ?Sized>(&self, rng: &mut R) -> Color {
        rng.random::<(u8, u8, u8)>().into()
    }
}

impl From<(u8, u8, u8)> for Color {
    fn from(value: (u8, u8, u8)) -> Self {
        Self(value.0, value.1, value.2)
    }
}

impl From<[u8; 3]> for Color {
    fn from(value: [u8; 3]) -> Self {
        Self(value[0], value[1], value[2])
    }
}

impl From<Color> for Vec<u8> {
    fn from(value: Color) -> Vec<u8> {
        vec![value.0, value.1, value.2]
    }
}

impl TryFrom<Vec<u8>> for Color {
    type Error = &'static str;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        if value.len() != 3 {
            Err("must have 3 elements")
        } else {
            Ok(Color(value[0], value[1], value[2]))
        }
    }
}

pub trait Shader {
    fn fragment(&self, v: &Array1<f32>) -> (bool, Color);
}
