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
    uv: [Array1<f32>; 3],
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
            uv: core::array::from_fn(Array1::default),
            camera,
        }
    }
    pub fn vertex(&mut self, face: usize, vert: usize) -> Array1<f32> {
        let v: [f32; 3] = self.model.vertices()[self.model.faces()[face][vert]].into();
        let gl_position = self
            .camera
            .modelview_mat()
            .dot(&array![v[0], v[1], v[2], 1.0]);
        let n: [f32; 3] = self.model.normals()[self.model.face_normals()[face][vert]].into();
        self.varying_nrm[vert] = self
            .camera
            .modelview_mat()
            .dot(&array![n[0], n[1], n[2], 0.0])
            .slice_move(s![..3]);
        self.tri[vert] = gl_position.clone();

        self.uv[vert] = Array::from_vec(self.model.uvs()[self.model.face_uvs()[face][vert]].into());
        self.camera.persp_mat().dot(&gl_position)
    }
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }
}

fn remap(coord: &Array1<f32>, height: usize, width: usize) -> [usize; 2] {
    let u = coord[0];
    let v = coord[1];

    let u_ix = (width as f32 * u).round();
    let v_ix = (height as f32 * v).round();

    [u_ix as usize, v_ix as usize]
}

impl<'a> Shader for PhongShader<'a> {
    fn fragment(&self, bar: &Array1<f32>) -> (bool, Color) {
        const WHITE: [u8; 3] = [255u8, 255u8, 255u8];
        let uv = &self.uv[0] * bar[0] + &self.uv[1] * bar[1] + &self.uv[2] * bar[2];
        let diff_c = match self.model.diffuse_texture() {
            Some(tex) => {
                let (height, width, _channels) = tex.dim();
                let uv_coords = remap(&uv, height, width);
                tex.slice(s![height - uv_coords[1] - 1, uv_coords[0], ..]) //.clone().into_owned()
            }
            None => aview1(&WHITE),
        };
        let spec_c = match self.model.specular_texture() {
            Some(tex) => {
                let (height, width, _channels) = tex.dim();
                let uv_coords = remap(&uv, height, width);
                tex.slice(s![height - uv_coords[1] - 1, uv_coords[0], ..]) //.clone().into_owned()
            }
            None => aview1(&WHITE),
        };

        let n = match self.model.normals_texture() {
            Some(tex) => {
                // If there's texture-encoded normal data, use it.
                let (height, width, _channels) = tex.dim();
                let uv_coords = remap(&uv, height, width);

                let n_ = tex
                    .slice(s![height - uv_coords[1] - 1, uv_coords[0], ..])
                    .clone();
                normalized(n_.mapv(|x| (x as f32 * 2.0 / 255.0) - 1.0))
            }
            None => normalized(
                // If absent, interpolate normals from vertices.
                bar[0] * &self.varying_nrm[0]
                    + bar[1] * &self.varying_nrm[1]
                    + bar[2] * &self.varying_nrm[2],
            ),
        };
        // normalize(&mut n);
        // println!("{:?}", n);

        // let n = normalized(cross_prod(
        //     &(&self.tri[1] - &self.tri[0]),
        //     &(&self.tri[2] - &self.tri[0]),
        // ));
        let r = normalized(2.0 * &n * (&n).dot(&self.light) - &self.light);

        let cos_alpha = (&n).dot(&self.light);
        let cos_beta = r.dot(&normalized(array![-1.0, 0.0, 2.0]));

        let ambient = 0.4f32;
        let diffuse = cos_alpha.max(0.);
        let specular = cos_beta.max(0.).powi(50) * spec_c[0] as f32 / 255.0;

        let out_c = (diff_c.mapv(|x| x as f32) * (ambient + diffuse + specular))
            .mapv(|x| x.min(255.0) as u8);
        (false, out_c.to_vec().try_into().unwrap())

        // for channel in &mut c {
        //     *channel =
        //         ((*channel as f32) * (ambient + 0.4 * diffuse + 0.7 * specular).min(1.0)) as u8;
        // }

        // (false, c.into())
    }
}
