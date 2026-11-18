use ndarray::{Array1, Array2, array};

fn viewport(x: i32, y: i32, w: u32, h: u32) -> Array2<f32> {
    array![
        [w as f32 / 2.0, 0., 0., x as f32 + w as f32 / 2.0],
        [0., h as f32 / 2.0, 0., y as f32 + h as f32 / 2.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

pub struct Viewport {
    t: Array2<f32>,
}

impl Viewport {
    pub fn new(x: i32, y: i32, w: u32, h: u32) -> Self {
        Self { t: viewport(x, y, w, h) }
    }

    pub fn transform(&self, v: &Array1<f32>) -> Array1<f32> {
        self.t.dot(v)
    }
}
