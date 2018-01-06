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

    for stage in 1..stages + 1 {
        for i in 0..2i32.pow(stage - 1) {
            for j in 0..2i32.pow(stages - stage) {
                let n = (2i32.pow(stages - stage + 1) * i + j) as usize;
                let m = 2i32.pow(stages - stage) as usize + n;
                let r = (2i32.pow(stage - 1) * j) as f64;

                let a = x[n];
                let b = x[m];
                let c = Complex64::new(
                    ((2.0 * PI * r) / len as f64).cos(),
                    -((2.0 * PI * r) / len as f64).sin(),
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
    for stage in 1..stages + 1 {
        for i in 0..2i32.pow(stage - 1) {
            index[(2i32.pow(stage - 1) + i) as usize] =
                index[i as usize] + 2i32.pow(stages - stage) as usize;
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

pub fn apply_window<F: Fn(f64) -> f64>(mut data: Vec<f64>, f: F) -> Vec<f64> {
    let len = data.len();
    for i in 0..len {
        data[i] = data[i] * f(i as f64 / len as f64);
    }
    data
}

pub fn blackman_harris(x: f64) -> f64 {
    0.35875
        - 0.48829 * (2.0 * PI * x).cos()
        + 0.14128 * (4.0 * PI * x).cos()
        - 0.01168 * (6.0 * PI * x).cos()
}



pub struct BiQuadIIR {
    in1: f64,
    in2: f64,
    out1: f64,
    out2: f64,
    b0: f64,
    b1: f64,
    b2: f64,
    a0: f64,
    a1: f64,
    a2: f64,
}

impl BiQuadIIR {
    pub fn new(b0: f64, b1: f64, b2: f64, a0: f64, a1: f64, a2: f64) -> Self {
        BiQuadIIR {
            in1: 0.0,
            in2: 0.0,
            out1: 0.0,
            out2: 0.0,
            b0: b0,
            b1: b1,
            b2: b2,
            a0: a0,
            a1: a1,
            a2: a2,
        }
    }

    pub fn next(&mut self, input: f64) -> f64 {
        let output = self.b0/self.a0 * input
            + self.b1/self.a0 * self.in1
            + self.b2/self.a0 * self.in2
            - self.a1/self.a0 * self.out1
            - self.a2/self.a0 * self.out2;
        self.in2  = self.in1;
        self.in1  = input;
        self.out2 = self.out1;
        self.out1 = output;
        output
    }
}

impl Default for BiQuadIIR {
    fn default() -> BiQuadIIR {
        Self {
            in1: 0.0,
            in2: 0.0,
            out1: 0.0,
            out2: 0.0,
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a0: 1.0,
            a1: 0.0,
            a2: 0.0,
        }
    }
}
