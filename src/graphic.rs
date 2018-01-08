extern crate piston_window;
use self::piston_window::*;

use super::*;
use super::dsp::*;
use super::sample::*;

use std::sync::{Arc, Mutex};

const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
const GRAY: [f32; 4] = [0.5, 0.5, 0.5, 1.0];
const CYAN: [f32; 4] = [0.0, 1.0, 1.0, 1.0];

pub struct Oscillo {
    shared_data: Arc<Mutex<Vec<f64>>>,
    local_data: Vec<f64>,
    ptr: usize,
}
impl Oscillo {
    pub fn new(len: usize) -> Self {
        let data = Arc::new(Mutex::new(vec![0f64; len]));
        let data_move = data.clone();
        std::thread::spawn(move || {
            let mut window: PistonWindow = WindowSettings::new("Oscilloscope", [640, 480])
                .exit_on_esc(true)
                .build()
                .unwrap();
            while let Some(event) = window.next() {
                let data = { data_move.lock().unwrap().clone() };
                window.draw_2d(&event, |context, graphics| {
                    clear(BLACK, graphics);
                    for i in 0..data.len() - 1 {
                        line(
                            CYAN,
                            1.0,
                            [
                                i as f64,
                                (data[i] + 1.0) * 240.0,
                                (i + 1) as f64,
                                (data[i + 1] + 1.0) * 240.0,
                            ],
                            context.transform,
                            graphics,
                        );
                    }
                });
            }
        });
        Self {
            shared_data: data,
            local_data: vec![0f64; len],
            ptr: 0,
        }
    }
}
impl<T, Ctx> Element<T, Ctx> for Oscillo
where
    T: IntoSample<Stereo<f64>> + Copy,
{
    type Src = T;
    fn next(&mut self, sink: T, _ctx: &Ctx) -> T {
        self.local_data[self.ptr] = sink.into_sample().l;
        self.ptr += 1;
        if self.ptr >= self.local_data.len() {
            self.ptr = 0;
            self.shared_data
                .lock()
                .unwrap()
                .copy_from_slice(&self.local_data);
        }
        sink
    }
}

pub struct Spectrum {
    shared_data: Arc<Mutex<Vec<f64>>>,
    local_data: Vec<f64>,
    ptr: usize,
}
impl Spectrum {
    pub fn new(len: usize) -> Self {
        let data = Arc::new(Mutex::new(vec![0f64; len]));
        let data_move = data.clone();
        std::thread::spawn(move || {
            let mut window: PistonWindow = WindowSettings::new("Spectrum analyzer", [640, 480])
                .exit_on_esc(true)
                .build()
                .unwrap();
            while let Some(event) = window.next() {
                let data = { data_move.lock().unwrap().clone() };
                //let data = apply_window(data, blackman_harris);
                let data = fft(data);
                window.draw_2d(&event, |context, graphics| {
                    clear(BLACK, graphics);
                    for i in 1..len / 2 {
                        let d = data[i] / len as f64 * 2.0;
                        if d == 0.0 {
                            continue;
                        }
                        let db = d.log(10.0) * 20.0;
                        let y = -db * 4.8 + 10.0;
                        let f = 44100.0 * i as f64 / len as f64;
                        let x = (f.log(10.0) - 50f64.log(10.0)) * 220.0;
                        line(CYAN, 1.0, [x, y, x, 480.0], context.transform, graphics);
                    }
                    for i in 0..10 {
                        let db = -10.0 * i as f64;
                        let y = -db * 4.8 + 10.0;
                        line(GRAY, 1.0, [0.0, y, 640.0, y], context.transform, graphics);
                    }
                    for i in 2..5 {
                        let f = 10i32.pow(i) as f64;
                        let x = (f.log(10.0) - 50f64.log(10.0)) * 220.0;
                        line(GRAY, 1.0, [x, 0.0, x, 480.0], context.transform, graphics);
                    }
                });
            }
        });
        Self {
            shared_data: data,
            local_data: vec![0f64; len],
            ptr: 0,
        }
    }
}
impl<T, Ctx> Element<T, Ctx> for Spectrum
where
    T: IntoSample<Stereo<f64>> + Copy,
{
    type Src = T;
    fn next(&mut self, sink: T, _ctx: &Ctx) -> T {
        self.local_data[self.ptr] = sink.into_sample().l;
        self.ptr += 1;
        if self.ptr >= self.local_data.len() {
            self.ptr = 0;
            self.shared_data
                .lock()
                .unwrap()
                .copy_from_slice(&self.local_data);
        }
        sink
    }
}
