extern crate num;

use self::num::complex::*;

use std::f64::consts::PI;

pub fn fft(samples: Vec<f64>) -> Vec<f64> {
    let stages = (samples.len() as f64).log2().floor() as u32;
    let len = 2i32.pow(stages) as usize;

    let mut x = Vec::with_capacity(len);
    for i in 0..len {
        x.push(Complex64::new(samples[i], 0f64));
    }

    for stage in 1..stages+1 {
        for i in 0..2i32.pow(stage - 1) {
            for j in 0..2i32.pow(stages - stage) {
                let n = (2i32.pow(stages - stage + 1) * i + j) as usize;
                let m = 2i32.pow(stages - stage) as usize + n;
                let r = (2i32.pow(stage - 1) * j) as f64;

                let a = x[n];
                let b = x[m];
                let c = Complex64::new(
                    ((2.0 * PI * r) / len as f64).cos(),
                    -((2.0 * PI * r) / len as f64).sin()
                );

                if stage < stages {
                    x[n] = a + b;
                    x[m] = (a - b) * c;
                } else {
                    x[n] = a + b;
                    x[m] = a - b;
                }
            }
        }
    }

    let mut index = vec![0 as usize; len];
    for stage in 1..stages+1 {
        for i in 0..2i32.pow(stage - 1) {
            index[(2i32.pow(stage - 1) + i) as usize] = index[i as usize] + 2i32.pow(stages - stage) as usize;
        }
    }

    for k in 0..len {
        if index[k] > k {
            let buf = x[index[k]];
            x[index[k]] = x[k];
            x[k] = buf;
        }
    }

    x.into_iter().map(|a| a.norm()).collect()
}

pub fn blackman_harris(x: f64) -> f64 {
    0.35875
        - 0.48829 * (2.0 * PI * x).cos()
        + 0.14128 * (4.0 * PI * x).cos()
        - 0.01168 * (6.0 * PI * x).cos()
}

