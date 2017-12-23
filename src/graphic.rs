extern crate piston_window;
use self::piston_window::*;

use super::*;
use super::dsp::*;
use super::sample::*;

use std::sync::{Arc, Mutex};

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
                    clear([0.0, 0.0, 0.0, 1.0], graphics);
                    for i in 0..data.len() - 1 {
                        line(
                            [0.0, 1.0, 1.0, 1.0],
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
                let data = apply_window(data, blackman_harris);
                let data = fft(data);
                window.draw_2d(&event, |context, graphics| {
                    clear([0.0, 0.0, 0.0, 1.0], graphics);
                    for i in 0..len / 2 {
                        let d = data[i];
                        if d == 0.0 { continue; }
                        let f = 44100.0 * i as f64 / len as f64;
                        let x = f.log(10.0) * 200.0 - 280.0;
                        line(
                            [0.0, 1.0, 1.0, 1.0],
                            1.0,
                            [
                                x, 480.0 - (d.log(10.0) * 200.0),
                                x, 480.0,
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