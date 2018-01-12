extern crate piston_window;
use self::piston_window::*;

use super::*;
use super::dsp::*;
use super::sample::*;

use std::sync::{Arc, Mutex};

const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
const GRAY: [f32; 4] = [0.5, 0.5, 0.5, 0.5];

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
                let width = window.size().width as f64;
                let height = window.size().height as f64;
                window.draw_2d(&event, |context, graphics| {
                    let data = { data_move.lock().unwrap().clone() };
                    clear(WHITE, graphics);
                    for i in 0..len - 1 {
                        line(
                            BLACK,
                            1.0,
                            [
                                i as f64 * width / len as f64,
                                (data[i] + 1.0) * height / 2.0,
                                (i + 1) as f64 * width / len as f64,
                                (data[i + 1] + 1.0) * height / 2.0,
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
                let width = window.size().width as f64;
                let height = window.size().height as f64;
                window.draw_2d(&event, |context, graphics| {
                    let data = { data_move.lock().unwrap().clone() };
                    //let data = apply_window(data, blackman_harris);
                    let data = fft(data);
                    clear(WHITE, graphics);
                    for i in 1..len / 2 {
                        let d = data[i] / len as f64 * 2.0;
                        if d == 0.0 {
                            continue;
                        }
                        let db = d.log(10.0) * 20.0;
                        let y = ((-db + 5.0) / 70.0) * height;
                        let f = 44100.0 * i as f64 / len as f64;
                        let x = (f.log(10.0) - 50f64.log(10.0)) * 0.35 * width;
                        line(BLACK, 1.0, [x, y, x, height], context.transform, graphics);
                    }
                    for i in 0..10 {
                        let db = -10.0 * i as f64;
                        let y = ((-db + 5.0) / 70.0) * height;
                        line(GRAY, 1.0, [0.0, y, width, y], context.transform, graphics);
                    }
                    for i in 1..5 {
                        for j in 1..11 {
                            let f = (10i32.pow(i) * j) as f64;
                            let x = (f.log(10.0) - 50f64.log(10.0)) * 0.35 * width;
                            line(GRAY, 1.0, [x, 0.0, x, height], context.transform, graphics);
                        }
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
