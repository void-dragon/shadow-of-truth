use std::sync::{Arc, Mutex};
use std::rc::Rc;
use std::cell::RefCell;

use crate::methatron::math::vector::{self, Vector, VectorUserData};

pub type Matrix = Arc<Mutex<[f32; 16]>>;

pub fn new() -> Matrix {
  let m = [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0];

  Arc::new(Mutex::new(m))
}

pub fn identity(m: &mut [f32; 16]) {
  m[0] = 1.0;
  m[1] = 0.0;
  m[2] = 0.0;
  m[3] = 0.0;
  m[4] = 0.0;
  m[5] = 1.0;
  m[6] = 0.0;
  m[7] = 0.0;
  m[8] = 0.0;
  m[9] = 0.0;
  m[10] = 1.0;
  m[11] = 0.0;
  m[12] = 0.0;
  m[13] = 0.0;
  m[14] = 0.0;
  m[15] = 1.0;
}

pub fn scale(m: &mut [f32; 16], v: &Vector) {
  m[0] = v[0] * m[0];
  m[1] = v[0] * m[1];
  m[2] = v[0] * m[2];
  m[3] = v[0] * m[3];

  m[4] = v[1] * m[4];
  m[5] = v[1] * m[5];
  m[6] = v[1] * m[6];
  m[7] = v[1] * m[7];

  m[8] = v[2] * m[8];
  m[9] = v[2] * m[9];
  m[10] = v[2] * m[10];
  m[11] = v[2] * m[11];
}

pub fn translate(m: &mut [f32; 16], v: &Vector) {
  m[12] = v[0] * m[0] + v[1] * m[4] + v[2] * m[8] + m[12];
  m[13] = v[0] * m[1] + v[1] * m[5] + v[2] * m[9] + m[13];
  m[14] = v[0] * m[2] + v[1] * m[6] + v[2] * m[10] + m[14];
  m[15] = v[0] * m[3] + v[1] * m[7] + v[2] * m[11] + m[15];
}

pub fn rotate_by_vector(m: &mut [f32; 16], a: f32, n: &Vector) {
  let sin_a = a.sin();
  let cos_a = a.cos();
  let mut tm = vec![0.0; 12];

  let _0 = cos_a + n[0] * n[0] * (1.0 - cos_a);
  let _4 = n[0] * n[1] * (1.0 - cos_a) - n[2] * sin_a;
  let _8 = n[0] * n[2] * (1.0 - cos_a) + n[1] * sin_a;

  let _1 = n[0] * n[1] * (1.0 - cos_a) + n[2] * sin_a;
  let _5 = cos_a + n[1] * n[1] * (1.0 - cos_a);
  let _9 = n[1] * n[2] * (1.0 - cos_a) - n[0] * sin_a;

  let _2 = n[0] * n[2] * (1.0 - cos_a) - n[1] * sin_a;
  let _6 = n[1] * n[2] * (1.0 - cos_a) + n[0] * sin_a;
  let _10 = cos_a + n[2] * n[2] * (1.0 - cos_a);

  tm[0] = _0 * m[0] + _1 * m[4] + _2 * m[8];
  tm[1] = _0 * m[1] + _1 * m[5] + _2 * m[9];
  tm[2] = _0 * m[2] + _1 * m[6] + _2 * m[10];
  tm[3] = _0 * m[3] + _1 * m[7] + _2 * m[11];

  tm[4] = _4 * m[0] + _5 * m[4] + _6 * m[8];
  tm[5] = _4 * m[1] + _5 * m[5] + _6 * m[9];
  tm[6] = _4 * m[2] + _5 * m[6] + _6 * m[10];
  tm[7] = _4 * m[3] + _5 * m[7] + _6 * m[11];

  tm[8] = _8 * m[0] + _9 * m[4] + _10 * m[8];
  tm[9] = _8 * m[1] + _9 * m[5] + _10 * m[9];
  tm[10] = _8 * m[2] + _9 * m[6] + _10 * m[10];
  tm[11] = _8 * m[3] + _9 * m[7] + _10 * m[11];

  for i in 0..12 {
    m[i] = tm[i];
  }
}

pub fn mul(m: &[f32; 16], o: &[f32; 16]) -> [f32; 16] {
  let mut e = [0.0; 16];

  e[0] = o[0] * m[0] + o[1] * m[4] + o[2] * m[8] + o[3] * m[12];
  e[1] = o[0] * m[1] + o[1] * m[5] + o[2] * m[9] + o[3] * m[13];
  e[2] = o[0] * m[2] + o[1] * m[6] + o[2] * m[10] + o[3] * m[14];
  e[3] = o[0] * m[3] + o[1] * m[7] + o[2] * m[11] + o[3] * m[15];

  e[4] = o[4] * m[0] + o[5] * m[4] + o[6] * m[8] + o[7] * m[12];
  e[5] = o[4] * m[1] + o[5] * m[5] + o[6] * m[9] + o[7] * m[13];
  e[6] = o[4] * m[2] + o[5] * m[6] + o[6] * m[10] + o[7] * m[14];
  e[7] = o[4] * m[3] + o[5] * m[7] + o[6] * m[11] + o[7] * m[15];

  e[8] = o[8] * m[0] + o[9] * m[4] + o[10] * m[8] + o[11] * m[12];
  e[9] = o[8] * m[1] + o[9] * m[5] + o[10] * m[9] + o[11] * m[13];
  e[10] = o[8] * m[2] + o[9] * m[6] + o[10] * m[10] + o[11] * m[14];
  e[11] = o[8] * m[3] + o[9] * m[7] + o[10] * m[11] + o[11] * m[15];

  e[12] = o[12] * m[0] + o[13] * m[4] + o[14] * m[8] + o[15] * m[12];
  e[13] = o[12] * m[1] + o[13] * m[5] + o[14] * m[9] + o[15] * m[13];
  e[14] = o[12] * m[2] + o[13] * m[6] + o[14] * m[10] + o[15] * m[14];
  e[15] = o[12] * m[3] + o[13] * m[7] + o[14] * m[11] + o[15] * m[15];

  e
}

pub fn mul_assign(m: &mut [f32; 16], o: &[f32; 16]) {
  let mut tm = [0.0; 16];

  tm[0] = o[0] * m[0] + o[1] * m[4] + o[2] * m[8] + o[3] * m[12];
  tm[1] = o[0] * m[1] + o[1] * m[5] + o[2] * m[9] + o[3] * m[13];
  tm[2] = o[0] * m[2] + o[1] * m[6] + o[2] * m[10] + o[3] * m[14];
  tm[3] = o[0] * m[3] + o[1] * m[7] + o[2] * m[11] + o[3] * m[15];

  tm[4] = o[4] * m[0] + o[5] * m[4] + o[6] * m[8] + o[7] * m[12];
  tm[5] = o[4] * m[1] + o[5] * m[5] + o[6] * m[9] + o[7] * m[13];
  tm[6] = o[4] * m[2] + o[5] * m[6] + o[6] * m[10] + o[7] * m[14];
  tm[7] = o[4] * m[3] + o[5] * m[7] + o[6] * m[11] + o[7] * m[15];

  tm[8] = o[8] * m[0] + o[9] * m[4] + o[10] * m[8] + o[11] * m[12];
  tm[9] = o[8] * m[1] + o[9] * m[5] + o[10] * m[9] + o[11] * m[13];
  tm[10] = o[8] * m[2] + o[9] * m[6] + o[10] * m[10] + o[11] * m[14];
  tm[11] = o[8] * m[3] + o[9] * m[7] + o[10] * m[11] + o[11] * m[15];

  tm[12] = o[12] * m[0] + o[13] * m[4] + o[14] * m[8] + o[15] * m[12];
  tm[13] = o[12] * m[1] + o[13] * m[5] + o[14] * m[9] + o[15] * m[13];
  tm[14] = o[12] * m[2] + o[13] * m[6] + o[14] * m[10] + o[15] * m[14];
  tm[15] = o[12] * m[3] + o[13] * m[7] + o[14] * m[11] + o[15] * m[15];

  for i in 0..16 {
    m[i] = tm[i];
  }
}

pub fn rotate_x(m: &mut [f32; 16], v: f32) {
  let sin_a = v.sin();
  let cos_a = v.cos();

  let _0 = cos_a + (1.0 - cos_a);
  let _5 = cos_a;
  let _6 = sin_a;
  let _9 = -sin_a;
  let _10 = cos_a;

  let tm_0 = _0 * m[0];
  let tm_1 = _0 * m[1];
  let tm_2 = _0 * m[2];
  let tm_3 = _0 * m[3];

  let tm_4 = _5 * m[4] + _6 * m[8];
  let tm_5 = _5 * m[5] + _6 * m[9];
  let tm_6 = _5 * m[6] + _6 * m[10];
  let tm_7 = _5 * m[7] + _6 * m[11];

  let tm_8 = _9 * m[4] + _10 * m[8];
  let tm_9 = _9 * m[5] + _10 * m[9];
  let tm_10 = _9 * m[6] + _10 * m[10];
  let tm_11 = _9 * m[7] + _10 * m[11];

  m[0] = tm_0;
  m[1] = tm_1;
  m[2] = tm_2;
  m[3] = tm_3;
  m[4] = tm_4;
  m[5] = tm_5;
  m[6] = tm_6;
  m[7] = tm_7;
  m[8] = tm_8;
  m[9] = tm_9;
  m[10] = tm_10;
  m[11] = tm_11;
}

pub fn rotate_y(m: &mut [f32; 16], v: f32) {
  let sin_a = v.sin();
  let cos_a = v.cos();

  let _0 = cos_a;
  let _8 = sin_a;
  let _2 = -sin_a;
  let _10 = cos_a;

  let tm_0 = _0 * m[0] + _2 * m[8];
  let tm_1 = _0 * m[1] + _2 * m[9];
  let tm_2 = _0 * m[2] + _2 * m[10];
  let tm_3 = _0 * m[3] + _2 * m[11];

  let tm_8 = _8 * m[0] + _10 * m[8];
  let tm_9 = _8 * m[1] + _10 * m[9];
  let tm_10 = _8 * m[2] + _10 * m[10];
  let tm_11 = _8 * m[3] + _10 * m[11];

  m[0] = tm_0;
  m[1] = tm_1;
  m[2] = tm_2;
  m[3] = tm_3;
  m[8] = tm_8;
  m[9] = tm_9;
  m[10] = tm_10;
  m[11] = tm_11;
}

pub fn rotate_z(m: &mut [f32; 16], v: f32) {
  let sin_a = v.sin();
  let cos_a = v.cos();

  let _0 = cos_a;
  let _4 = -sin_a;
  let _1 = sin_a;
  let _5 = cos_a;

  let tm_0 = _0 * m[0] + _1 * m[4];
  let tm_1 = _0 * m[1] + _1 * m[5];
  let tm_2 = _0 * m[2] + _1 * m[6];
  let tm_3 = _0 * m[3] + _1 * m[7];

  let tm_4 = _4 * m[0] + _5 * m[4];
  let tm_5 = _4 * m[1] + _5 * m[5];
  let tm_6 = _4 * m[2] + _5 * m[6];
  let tm_7 = _4 * m[3] + _5 * m[7];

  m[0] = tm_0;
  m[1] = tm_1;
  m[2] = tm_2;
  m[3] = tm_3;
  m[4] = tm_4;
  m[5] = tm_5;
  m[6] = tm_6;
  m[7] = tm_7;
}

pub fn look_at(m: &mut [f32; 16], lookat: &Vector, up: &Vector) {
  let eye = [m[12], m[13], m[14]];
  let mut f = vector::sub(&eye, &lookat);

  vector::normalize(&mut f);

  let mut s = vector::cross(&f, &up);
  vector::normalize(&mut s);

  let mut u = vector::cross(&s, &f);
  vector::normalize(&mut u);

  m[0] = s[0];
  m[1] = s[1];
  m[2] = s[2];
  m[4] = u[0];
  m[5] = u[1];
  m[6] = u[2];
  m[8] = f[0];
  m[9] = f[1];
  m[10] = f[2];
}

pub fn look_at_from(eye: &[f32; 3], lookat: &Vector, up: &Vector) -> [f32; 16] {
  let mut m = [0.0; 16];

  let mut f = vector::sub(&eye, &lookat);
  vector::normalize(&mut f);

  let mut s = vector::cross(&f, &up);
  vector::normalize(&mut s);

  let mut u = vector::cross(&s, &f);
  vector::normalize(&mut u);

  m[0] = s[0];
  m[1] = s[1];
  m[2] = s[2];
  m[3] = 0.0; //-vector::dot(&s, &eye);
  m[4] = u[0];
  m[5] = u[1];
  m[6] = u[2];
  m[7] = 0.0; //-vector::dot(&u, &eye);
  m[8] = f[0];
  m[9] = f[1];
  m[10] = f[2];
  m[11] = 0.0; //-vector::dot(&f, &eye);
  m[12] = 0.0; //-vector::dot(&s, &eye);
  m[13] = 0.0; //-vector::dot(&u, &eye);
  m[14] = 0.0; //-vector::dot(&f, &eye);
  m[15] = 1.0;

  m
}

pub fn determinant(m: &[f32; 16]) -> f32 {
  let erg = m[12] * m[9] * m[6] * m[3] - m[8] * m[13] * m[6] * m[3] - m[12] * m[5] * m[10] * m[3] + m[4] * m[13] * m[10] * m[3] + m[8] * m[5] * m[14] * m[3]
    - m[4] * m[9] * m[14] * m[3]
    - m[12] * m[9] * m[2] * m[7]
    + m[8] * m[13] * m[2] * m[7]
    + m[12] * m[1] * m[10] * m[7]
    - m[0] * m[13] * m[10] * m[7]
    - m[8] * m[1] * m[14] * m[7]
    + m[0] * m[9] * m[14] * m[7]
    + m[12] * m[5] * m[2] * m[11]
    - m[4] * m[13] * m[2] * m[11]
    - m[12] * m[1] * m[6] * m[11]
    + m[0] * m[13] * m[6] * m[11]
    + m[4] * m[1] * m[14] * m[11]
    - m[0] * m[5] * m[14] * m[11]
    - m[8] * m[5] * m[2] * m[15]
    + m[4] * m[9] * m[2] * m[15]
    + m[8] * m[1] * m[6] * m[15]
    - m[0] * m[9] * m[6] * m[15]
    - m[4] * m[1] * m[10] * m[15]
    + m[0] * m[5] * m[10] * m[15];

  return erg;
}

pub fn distance_to_vector(m: &[f32; 16], v: &Vector) -> f32 {
  let dx = m[12] - v[0];
  let dy = m[13] - v[1];
  let dz = m[14] - v[2];

  return (dx * dx + dy * dy + dz * dz).sqrt();
}

pub fn distance_to_matrix(m: &[f32; 16], o: &[f32; 16]) -> f32 {
  let dx = m[12] - o[12];
  let dy = m[13] - o[13];
  let dz = m[14] - o[14];

  return (dx * dx + dy * dy + dz * dz).sqrt();
}

pub fn rotation(m: &[f32; 16]) -> [f32; 4] {
  let tr = m[0] + m[5] + m[10];
  let mut qw = 0.0;
  let mut qx = 0.0;
  let mut qy = 0.0;
  let mut qz = 0.0;

  if tr > 0.0 {
    let s = (tr + 1.0).sqrt() * 2.0; // S=4*qw
    qw = 0.25 * s;
    qx = (m[6] - m[9]) / s;
    qy = (m[8] - m[2]) / s;
    qz = (m[1] - m[4]) / s;
  } else if (m[0] > m[5]) & (m[0] > m[10]) {
    let s = (1.0 + m[0] - m[5] - m[10]).sqrt() * 2.0; // S=4*qx
    qw = (m[6] - m[9]) / s;
    qx = 0.25 * s;
    qy = (m[4] + m[1]) / s;
    qz = (m[8] + m[2]) / s;
  } else if m[5] > m[10] {
    let s = (1.0 + m[5] - m[0] - m[10]).sqrt() * 2.0; // S=4*qy
    qw = (m[8] - m[2]) / s;
    qx = (m[4] + m[1]) / s;
    qy = 0.25 * s;
    qz = (m[9] + m[6]) / s;
  } else {
    let s = (1.0 + m[10] - m[0] - m[5]).sqrt() * 2.0; // S=4*qz
    qw = (m[1] - m[4]) / s;
    qx = (m[8] + m[2]) / s;
    qy = (m[9] + m[6]) / s;
    qz = 0.25 * s;
  }

  return [qx, qy, qz, qw];
}

pub fn inverse(m: &[f32; 16], res: &mut [f32; 16]) {
  let d = determinant(m);

  res[0] = (-m[13] * m[10] * m[7] + m[9] * m[14] * m[7] + m[13] * m[6] * m[11] - m[5] * m[14] * m[11] - m[9] * m[6] * m[15] + m[5] * m[10] * m[15]) / d;
  res[4] = (m[12] * m[10] * m[7] - m[8] * m[14] * m[7] - m[12] * m[6] * m[11] + m[4] * m[14] * m[11] + m[8] * m[6] * m[15] - m[4] * m[10] * m[15]) / d;
  res[8] = (-m[12] * m[9] * m[7] + m[8] * m[13] * m[7] + m[12] * m[5] * m[11] - m[4] * m[13] * m[11] - m[8] * m[5] * m[15] + m[4] * m[9] * m[15]) / d;
  res[12] = (m[12] * m[9] * m[6] - m[8] * m[13] * m[6] - m[12] * m[5] * m[10] + m[4] * m[13] * m[10] + m[8] * m[5] * m[14] - m[4] * m[9] * m[14]) / d;
  res[1] = (m[13] * m[10] * m[3] - m[9] * m[14] * m[3] - m[13] * m[2] * m[11] + m[1] * m[14] * m[11] + m[9] * m[2] * m[15] - m[1] * m[10] * m[15]) / d;
  res[5] = (-m[12] * m[10] * m[3] + m[8] * m[14] * m[3] + m[12] * m[2] * m[11] - m[0] * m[14] * m[11] - m[8] * m[2] * m[15] + m[0] * m[10] * m[15]) / d;
  res[9] = (m[12] * m[9] * m[3] - m[8] * m[13] * m[3] - m[12] * m[1] * m[11] + m[0] * m[13] * m[11] + m[8] * m[1] * m[15] - m[0] * m[9] * m[15]) / d;
  res[13] = (-m[12] * m[9] * m[2] + m[8] * m[13] * m[2] + m[12] * m[1] * m[10] - m[0] * m[13] * m[10] - m[8] * m[1] * m[14] + m[0] * m[9] * m[14]) / d;
  res[2] = (-m[13] * m[6] * m[3] + m[5] * m[14] * m[3] + m[13] * m[2] * m[7] - m[1] * m[14] * m[7] - m[5] * m[2] * m[15] + m[1] * m[6] * m[15]) / d;
  res[6] = (m[12] * m[6] * m[3] - m[4] * m[14] * m[3] - m[12] * m[2] * m[7] + m[0] * m[14] * m[7] + m[4] * m[2] * m[15] - m[0] * m[6] * m[15]) / d;
  res[10] = (-m[12] * m[5] * m[3] + m[4] * m[13] * m[3] + m[12] * m[1] * m[7] - m[0] * m[13] * m[7] - m[4] * m[1] * m[15] + m[0] * m[5] * m[15]) / d;
  res[14] = (m[12] * m[5] * m[2] - m[4] * m[13] * m[2] - m[12] * m[1] * m[6] + m[0] * m[13] * m[6] + m[4] * m[1] * m[14] - m[0] * m[5] * m[14]) / d;
  res[3] = (m[9] * m[6] * m[3] - m[5] * m[10] * m[3] - m[9] * m[2] * m[7] + m[1] * m[10] * m[7] + m[5] * m[2] * m[11] - m[1] * m[6] * m[11]) / d;
  res[7] = (-m[8] * m[6] * m[3] + m[4] * m[10] * m[3] + m[8] * m[2] * m[7] - m[0] * m[10] * m[7] - m[4] * m[2] * m[11] + m[0] * m[6] * m[11]) / d;
  res[11] = (m[8] * m[5] * m[3] - m[4] * m[9] * m[3] - m[8] * m[1] * m[7] + m[0] * m[9] * m[7] + m[4] * m[1] * m[11] - m[0] * m[5] * m[11]) / d;
  res[15] = (-m[8] * m[5] * m[2] + m[4] * m[9] * m[2] + m[8] * m[1] * m[6] - m[0] * m[9] * m[6] - m[4] * m[1] * m[10] + m[0] * m[5] * m[10]) / d;
}

pub fn perspective(m: &mut [f32; 16], fovy: f32, aspect: f32, znear: f32, zfar: f32) {
  // float f = 1 / tanf(fovy / 2);
  let f = (fovy * 0.5).cos() / (fovy * 0.5).sin(); // numerical more stable

  m[0] = f / aspect;
  m[1] = 0.0;
  m[2] = 0.0;
  m[3] = 0.0;
  m[4] = 0.0;
  m[5] = f;
  m[6] = 0.0;
  m[7] = 0.0;
  m[8] = 0.0;
  m[9] = 0.0;
  m[10] = (zfar + znear) / (znear - zfar);
  m[11] = -1.0;
  m[12] = 0.0;
  m[13] = 0.0;
  m[14] = (2.0 * znear * zfar) / (znear - zfar);
  m[15] = 1.0;
}

pub fn ortho(top: f32, bottom: f32, left: f32, right: f32, near: f32, far: f32) -> [f32; 16] {
  let mut m = [0.0; 16];

  m[0] = 2.0 / (right - left);
  m[1] = 0.0;
  m[2] = 0.0;
  m[3] = 0.0;
  m[4] = 0.0;
  m[5] = 2.0 / (top - bottom);
  m[6] = 0.0;
  m[7] = 0.0;
  m[8] = 0.0;
  m[9] = 0.0;
  m[10] = 2.0 / (far - near);
  m[11] = 0.0;
  m[12] = -(right + left) / (right - left);
  m[13] = -(top + bottom) / (top - bottom);
  m[14] = -(far + near) / (far - near);
  m[15] = 1.0;

  m
}

pub struct MatrixUserData {
  pub matrix: Matrix,
}

impl mlua::UserData for MatrixUserData {
  fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
    methods.add_method("identity", |_, this, (): ()| {
      let mut m = this.matrix.lock().unwrap();
      identity(&mut *m);
      Ok(())
    });

    methods.add_method("look_at", |_, this, (look, up): (mlua::AnyUserData, mlua::Value)| {
      let l = look.borrow::<MatrixUserData>().unwrap();
      let mut m = this.matrix.lock().unwrap();
      let lm = l.matrix.lock().unwrap();
      let v = [lm[12], lm[13], lm[14]];

      look_at(&mut m, &v, &[0.0, 1.0, 0.0]);

      Ok(())
    });

    methods.add_method("position", |_, this, (): ()| {
      let m = this.matrix.lock().unwrap();
      let v: Vec<f32> = vec![m[12], m[13], m[14]];
      Ok(v)
    });

    methods.add_method("translate", |_, this, v: Vec<f32>| {
      let mut m = this.matrix.lock().unwrap();
      let vt = [v[0], v[1], v[2]];
      translate(&mut *m, &vt);

      Ok(())
    });

    methods.add_method("rotate_x", |_, this, a: f32| {
      let mut m = this.matrix.lock().unwrap();
      rotate_x(&mut *m, a);
      Ok(())
    });

    methods.add_method("rotate_y", |_, this, a: f32| {
      let mut m = this.matrix.lock().unwrap();
      rotate_y(&mut *m, a);
      Ok(())
    });

    methods.add_method("rotate_z", |_, this, a: f32| {
      let mut m = this.matrix.lock().unwrap();
      rotate_z(&mut *m, a);
      Ok(())
    });

    methods.add_method("scale", |_, this, a: f32| {
      let mut m = this.matrix.lock().unwrap();
      scale(&mut *m, &[a, a, a]);
      Ok(())
    });

    methods.add_method("batch", |_, this, cb: mlua::Function| {
      let mut m = this.matrix.lock().unwrap();
      {
        let data = Rc::new(RefCell::new(m.clone()));
        let local = LocalMatrixData(data.clone());
        cb.call((local,))?;
        *m = *data.borrow();
      }
      Ok(())
    });
  }
}

struct LocalMatrixData(Rc<RefCell<[f32; 16]>>);

impl mlua::UserData for LocalMatrixData {
  fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {

    methods.add_method("identity", |_, this, (): ()| {
      identity(&mut *this.0.borrow_mut());
      Ok(())
    });

    methods.add_method("translate", |_, this, v: Vec<f32>| {
      let vt = [v[0], v[1], v[2]];
      translate(&mut *this.0.borrow_mut(), &vt);

      Ok(())
    });

    methods.add_method("rotate_x", |_, this, a: f32| {
      rotate_x(&mut *this.0.borrow_mut(), a);
      Ok(())
    });

    methods.add_method("rotate_y", |_, this, a: f32| {
      rotate_y(&mut *this.0.borrow_mut(), a);
      Ok(())
    });

    methods.add_method("rotate_z", |_, this, a: f32| {
      rotate_z(&mut *this.0.borrow_mut(), a);
      Ok(())
    });
  }
}

pub fn load_module(lua: &mlua::Lua, ns: &mlua::Table) -> Result<(), mlua::Error> {
  let module = lua.create_table()?;

  ns.set("matrix", module)?;

  Ok(())
}
