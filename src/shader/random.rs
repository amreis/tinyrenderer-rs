use ndarray::prelude::*;

use crate::shader::shader::Color;
use crate::{camera::Camera, objmodel::Model, shader::shader::Shader};

pub struct RandomShader<'a> {
    model: &'a Model,
    color: Color,
    tri: Vec<Array1<f32>>,
    camera: &'a Camera,
}

impl<'a> RandomShader<'a> {
    pub fn new(model: &'a Model, camera: &'a Camera) -> Self {
        Self {
            model,
            color: Color::default(),
            tri: vec![Array1::default(3); 3],
            camera,
        }
    }

    pub fn vertex(&mut self, face: usize, vert: usize) -> Array1<f32> {
        let v: [f32; 3] =
            self.model.vertices()[Into::<[_; 3]>::into(self.model.faces()[face])[vert]].into();
        let gl_position = self
            .camera
            .modelview_mat()
            .dot(&array![v[0], v[1], v[2], 1.0]);
        self.tri[vert] = gl_position.iter().copied().collect();
        self.camera.persp_mat().dot(&gl_position)
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }
}

impl<'a> Shader for RandomShader<'a> {
    fn fragment(&self, v: &Array1<f32>) -> (bool, Color) {
        (false, self.color)
    }
}
