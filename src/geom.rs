use crate::{shader, viewport};
use ndarray::{prelude::*, stack};

#[derive(Debug)]
pub struct Triangle {
    v1: Array1<f32>,
    v2: Array1<f32>,
    v3: Array1<f32>,
}

impl Triangle {
    pub fn new(v1: Array1<f32>, v2: Array1<f32>, v3: Array1<f32>) -> Self {
        Self { v1, v2, v3 }
    }

    pub fn rasterize<S>(
        mut self,
        viewport: &viewport::Viewport,
        shader: &S,
        framebuffer: &mut Array3<u8>,
        depthbuffer: &mut Array2<f32>,
    ) where
        S: shader::Shader,
    {
        self.v1 /= self.v1[3];
        self.v2 /= self.v2[3];
        self.v3 /= self.v3[3];

        let a_i = viewport.transform(&self.v1).round().mapv(|x| x as i32);
        let b_i = viewport.transform(&self.v2).round().mapv(|x| x as i32);
        let c_i = viewport.transform(&self.v3).round().mapv(|x| x as i32);

        let m = stack![Axis(0), a_i, b_i, c_i];
        let (xmin, xmax) = m
            .column(0)
            .iter()
            .fold((i32::MAX, i32::MIN), |(curmin, curmax), &elem| {
                (curmin.min(elem), curmax.max(elem))
            });
        // let &xmax = m.column(0).iter().max().unwrap();
        let (ymin, ymax) = m
            .column(1)
            .iter()
            .fold((i32::MAX, i32::MIN), |(curmin, curmax), &elem| {
                (curmin.min(elem), curmax.max(elem))
            });

        let ab_x = b_i[0] - a_i[0];
        let ab_y = b_i[1] - a_i[1];
        let bc_x = c_i[0] - b_i[0];
        let bc_y = c_i[1] - b_i[1];

        let height = depthbuffer.len_of(Axis(0));

        let total_area = ab_x * bc_y - ab_y * bc_x;
        if total_area < 1 {
            return;
        }

        // let adjugate = array![[by - cy, cx - bx], [cy - ay, ax - cx]];

        let [lbx, ubx, lby, uby] = [
            xmin.max(0) as usize,
            (xmax as usize).min(framebuffer.len_of(Axis(1)) - 1),
            ymin.max(0) as usize,
            (ymax as usize).min(framebuffer.len_of(Axis(0)) - 1),
        ];
        for x in lbx..=ubx {
            for y in lby..=uby {
                let y_ = y as i32;
                let x_ = x as i32;
                let area_opp_a =
                    (y_ - b_i[1]) * (c_i[0] - b_i[0]) - (x_ - b_i[0]) * (c_i[1] - b_i[1]);
                let area_opp_b =
                    (y_ - c_i[1]) * (a_i[0] - c_i[0]) - (x_ - c_i[0]) * (a_i[1] - c_i[1]);
                let area_opp_c = total_area - area_opp_a - area_opp_b;

                let has_neg = (area_opp_a < 0) || (area_opp_b < 0) || (area_opp_c < 0);
                if has_neg {
                    continue;
                }

                let alpha = area_opp_a as f32 / total_area as f32;
                let beta = area_opp_b as f32 / total_area as f32;
                let gamma = area_opp_c as f32 / total_area as f32;
                let depth = alpha * self.v1[2] + beta * self.v2[2] + gamma * self.v3[2];

                let Some(cur_depth) = depthbuffer.get_mut([height - y - 1, x]) else {
                    panic!("out of bounds")
                };
                if depth <= *cur_depth {
                    continue;
                }
                let (discard, color) = shader.fragment(&array![alpha, beta, gamma]);
                if discard {
                    continue;
                }
                let mut slice = framebuffer.slice_mut(s![height - y - 1, x, ..]);
                slice.assign(&Array::from_vec(color.into()));
                *cur_depth = depth;
            }
        }
    }
}
