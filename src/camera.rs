use ndarray::{prelude::*, stack};

use crate::linalg::{cross_prod, norm, normalized};

fn perspmat(f: f32) -> Array2<f32> {
    array![
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, -1. / f, 1.0]
    ]
}

pub enum Camera {
    Partial {
        model_view_t: Option<Array2<f32>>,
        perspective_t: Option<Array2<f32>>,
    },
    Full {
        model_view_t: Array2<f32>,
        perspective_t: Array2<f32>,
    },
    Compiled {
        full_t: Array2<f32>,
    },
}

impl Camera {
    pub fn new() -> Self {
        Camera::Partial {
            model_view_t: None,
            perspective_t: None,
        }
    }
    pub fn look_at(mut self, eye: &Array1<f32>, center: &Array1<f32>, up: &Array1<f32>) -> Camera {
        let n = normalized(eye - center);
        let l = normalized(cross_prod(up, &n));
        let m = normalized(cross_prod(&n, &l));

        let submat = stack![Axis(0), l, m, n];

        let mut firstblock = Array2::<f32>::eye(4);
        firstblock.slice_mut(s![0..3, 0..3]).assign(&submat);

        let mut translateblock = Array2::<f32>::eye(4);
        translateblock.slice_mut(s![0..3, 3]).assign(&-center);

        let computed = firstblock.dot(&translateblock);
        match self {
            Camera::Partial {
                ref mut model_view_t,
                perspective_t: None,
            } => {
                model_view_t.replace(computed);
                self
            }
            Camera::Partial {
                perspective_t: Some(mat),
                ..
            } => Camera::Full {
                model_view_t: computed,
                perspective_t: mat,
            },
            Camera::Full {
                ref mut model_view_t,
                ..
            } => {
                *model_view_t = computed;
                self
            }
            Camera::Compiled { .. } => Camera::Partial {
                model_view_t: Some(computed),
                perspective_t: None,
            },
        }
    }

    pub fn perspective(mut self, f: f32) -> Self {
        let persp = perspmat(f);
        match self {
            Camera::Partial {
                ref mut perspective_t,
                model_view_t: None,
            } => {
                perspective_t.replace(persp);
                self
            }
            Camera::Partial {
                model_view_t: Some(mat),
                ..
            } => Camera::Full {
                perspective_t: persp,
                model_view_t: mat,
            },
            Camera::Full {
                ref mut perspective_t,
                ..
            } => {
                *perspective_t = persp;
                self
            }
            Camera::Compiled { .. } => Camera::Partial {
                perspective_t: Some(persp),
                model_view_t: None,
            },
        }
    }

    pub fn compile(self) -> Self {
        match self {
            Camera::Full {
                model_view_t,
                perspective_t,
            } => Camera::Compiled {
                full_t: perspective_t.dot(&model_view_t),
            },
            Camera::Compiled { .. } => self,
            Camera::Partial { .. } => panic!("cannot compile partially specified camera"),
        }
    }

    pub fn transform(&self, v: &Array1<f32>) -> Array1<f32> {
        if let Camera::Partial { .. } = self {
            panic!("can't call transform without informing perspective and modelview")
        }

        match self {
            Camera::Compiled { full_t } => full_t.dot(v),
            Camera::Full {
                model_view_t,
                perspective_t,
            } => perspective_t.dot(&model_view_t.dot(v)),
            Camera::Partial { .. } => unreachable!(),
        }
    }

    pub fn modelview_mat(&self) -> &Array2<f32> {
        if let Camera::Full { model_view_t, .. } = self {
            model_view_t
        } else if let Camera::Partial {
            model_view_t: Some(t),
            ..
        } = self
        {
            &t
        } else {
            panic!()
        }
    }

    pub fn persp_mat(&self) -> &Array2<f32> {
        if let Camera::Full { perspective_t, .. } = self {
            perspective_t
        } else if let Camera::Partial {
            perspective_t: Some(t),
            ..
        } = self
        {
            &t
        } else {
            panic!()
        }
    }
}
