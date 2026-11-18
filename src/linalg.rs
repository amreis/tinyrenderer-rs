use ndarray::prelude::*;

pub fn cross_prod(u: &Array1<f32>, v: &Array1<f32>) -> Array1<f32> {
    debug_assert!(u.len() == v.len() && u.len() == 3);

    array![
        u[1] * v[2] - u[2] * v[1],
        u[2] * v[0] - u[0] * v[2],
        u[0] * v[1] - u[1] * v[0]
    ]
}

pub fn normalize(v: &mut Array1<f32>) {
    *v /= norm(v)
}

pub fn normalized(v: Array1<f32>) -> Array1<f32> {
    let n = norm(&v);
    v / n
}

pub fn norm(v: &Array1<f32>) -> f32 {
    (v * v).sum().sqrt()
}
