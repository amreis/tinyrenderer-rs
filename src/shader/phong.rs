use ndarray::prelude::*;

use crate::linalg::{cross_prod, normalize, normalized};
use crate::shader::shader::Color;
use crate::{camera::Camera, objmodel::Model, shader::shader::Shader};
pub struct PhongShader<'a> {
    model: &'a Model,
    light: Array1<f32>, // light coordinates
    color: Color,
    tri: [Array1<f32>; 3],
    varying_nrm: [Array1<f32>; 3],
    camera: &'a Camera,
}

impl<'a> PhongShader<'a> {
    pub fn new(model: &'a Model, light: Array1<f32>, camera: &'a Camera) -> Self {
        let light = normalized(
            camera
                .modelview_mat()
                .dot(&array![light[0], light[1], light[2], 0.0])
                .slice_move(s![0..3]),
        );

        Self {
            model,
            light,
            color: Color::default(),
            tri: core::array::from_fn(Array1::default),
            varying_nrm: core::array::from_fn(Array1::default),
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
        let n: [f32; 3] = self.model.normals()
            [Into::<[_; 3]>::into(self.model.face_normals()[face])[vert]]
            .into();
        self.varying_nrm[vert] = self
            .camera
            .modelview_mat()
            .dot(&array![n[0], n[1], n[2], 0.0]).slice_move(s![..3]);
        self.tri[vert] = gl_position.iter().copied().collect();
        self.camera.persp_mat().dot(&gl_position)
    }
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }
}

impl<'a> Shader for PhongShader<'a> {
    fn fragment(&self, bar: &Array1<f32>) -> (bool, Color) {
        let mut c = [255u8, 255u8, 255u8];
        let n = normalized(
            bar[0] * &self.varying_nrm[0]
                + bar[1] * &self.varying_nrm[1]
                + bar[2] * &self.varying_nrm[2],
        );
        // let n = normalized(cross_prod(
        //     &(&self.tri[1] - &self.tri[0]),
        //     &(&self.tri[2] - &self.tri[0]),
        // ));
        let r = normalized(2.0 * &n * (&n).dot(&self.light) - &self.light);

        let cos_alpha = (&n).dot(&self.light);
        let cos_beta = r.dot(&normalized(array![-1.0, 0.0, 2.0]));

        let ambient = 0.1f32;
        let diffuse = cos_alpha.max(0.);
        let specular = cos_beta.max(0.).powi(50);

        for channel in &mut c {
            *channel =
                ((*channel as f32) * (ambient + 0.4 * diffuse + 1.0 * specular).min(1.0)) as u8;
        }

        (false, c.into())
    }
}
