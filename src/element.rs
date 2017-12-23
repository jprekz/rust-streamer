extern crate cpal;

use super::*;
use super::wav::*;
use super::sample::*;

use std::fs::File;

// Input / Output

pub struct WAVSource<Src> {
    wav: WAV,
    pos: usize,
    src_type: ::std::marker::PhantomData<Src>
}
impl<Src> WAVSource<Src> {
    pub fn new(filename: &str) -> Self {
        let file = File::open(filename).unwrap();
        let wav = WAV::new(file);
        Self {
            wav: wav,
            pos: 0,
            src_type: ::std::marker::PhantomData
        }
    }
}
impl<Ctx, Src> Element<(), Ctx> for WAVSource<Src>
where Src: Sample,
      Src::Member: FromSampleType<i16> {
    type Src = Src;
    fn next(&mut self, _sink: (), _ctx: &Ctx) -> Src {
        self.pos += 1;
        self.wav.get_sample_as::<Src>(self.pos - 1).unwrap()
    }
}

pub struct CpalSink;
impl CpalSink {
    pub fn new() -> Self {
        CpalSink {}
    }
}
impl<S, Ctx> PullElement<S, Ctx> for CpalSink
where S: IntoSample<Stereo<f32>>,
      Ctx: FreqCtx + Sync {
    fn start<E>(&mut self, mut sink: E, ctx: &Ctx)
    where E: Element<(), Ctx, Src=S> + Send + Sync + 'static {
        use self::cpal::*;

        let endpoint = default_endpoint().expect("Failed to get default endpoint");
        let format = Format {
            channels: vec![ChannelPosition::FrontLeft, ChannelPosition::FrontRight],
            samples_rate: SamplesRate(ctx.get_freq()),
            data_type: SampleFormat::F32
        };
        let event_loop = EventLoop::new();
        let voice_id = event_loop.build_voice(&endpoint, &format).expect("Failed to build voice");
        event_loop.play(voice_id);
        event_loop.run(move |_, buffer| {
            match buffer {
                UnknownTypeBuffer::F32(mut buffer) => {
                    for sample in buffer.chunks_mut(format.channels.len()) {
                        let Stereo { l, r } = sink.next((), ctx).into_sample();
                        sample[0] = l;
                        sample[1] = r;
                    }
                },
                _ => panic!()
            };
        });
    }
    fn stop(&mut self) {
        unimplemented!();
    }
}

pub struct NullSink;
impl NullSink {
    pub fn new() -> Self {
        Self {}
    }
}
impl<T, Ctx> Element<T, Ctx> for NullSink {
    type Src = ();
    fn next(&mut self, _sink: T, _ctx: &Ctx) {
        // do nothing
    }
}

pub struct PrintSink;
impl PrintSink {
    pub fn new() -> Self {
        Self {}
    }
}
impl<T, Ctx> Element<T, Ctx> for PrintSink
where T: std::fmt::Debug {
    type Src = ();
    fn next(&mut self, sink: T, _ctx: &Ctx) {
        println!("{:?}", sink);
    }
}

// Common Element

pub struct Ident;
impl Ident {
    pub fn new() -> Self {
        Self{}
    }
}
impl<T, Ctx> Element<T, Ctx> for Ident {
    type Src = T;
    fn next(&mut self, sink: T, _ctx: &Ctx) -> T {
        sink
    }
}

pub struct Tee<F> {
    f: F
}
impl<F> Tee<F> {
    pub fn new(f: F) -> Self {
        Tee { f: f }
    }
}
impl<T, Ctx, F> Element<T, Ctx> for Tee<F>
where F: FnMut(T),
      T: Copy {
    type Src = T;
    fn next(&mut self, sink: T, _ctx: &Ctx) -> T {
        (self.f)(sink);
        sink
    }
}

pub struct FnElement<F> {
    f: F
}
impl<F> FnElement<F> {
    pub fn new(f: F) -> Self {
        FnElement { f: f }
    }
}
impl<T1, T2, Ctx, F> Element<T1, Ctx> for FnElement<F>
where F: FnMut(T1) -> T2,
      T1: Copy,
      T2: Copy {
    type Src = T2;
    fn next(&mut self, sink: T1, _ctx: &Ctx) -> T2 {
        (self.f)(sink)
    }
}



#[cfg(feature = "graphic")]
pub mod graphic {
    extern crate piston_window;
    use self::piston_window::*;

    use super::*;
    use super::sample::*;

    use std::sync::{Arc, Mutex};

    pub struct Oscillo {
        shared_data: Arc<Mutex<Vec<f64>>>,
        local_data: Vec<f64>,
        ptr: usize
    }
    impl Oscillo {
        pub fn new(len: usize) -> Self {
            let data = Arc::new(Mutex::new(vec![0f64; len]));
            let data_move = data.clone();
            std::thread::spawn(move || {
                let mut window: PistonWindow =
                    WindowSettings::new("Oscilloscope", [640, 480])
                    .exit_on_esc(true).build().unwrap();
                while let Some(event) = window.next() {
                    let data = {
                        data_move.lock().unwrap().clone()
                    };
                    window.draw_2d(&event, |context, graphics| {
                        clear([0.0, 0.0, 0.0, 1.0], graphics);
                        for i in 0 .. data.len() - 1 {
                            line([0.0, 1.0, 1.0, 1.0], 1.0,
                                [i as f64, (data[i] + 1.0) * 240.0,
                                 (i+1) as f64, (data[i+1] + 1.0) * 240.0],
                                context.transform, graphics);
                        }
                    });
                }
            });
            Self {
                shared_data: data,
                local_data: vec![0f64; len],
                ptr: 0
            }
        }
    }
    impl<T, Ctx> Element<T, Ctx> for Oscillo 
    where T: IntoSample<Stereo<f64>> + Copy {
        type Src = T;
        fn next(&mut self, sink: T, _ctx: &Ctx) -> T {
            self.local_data[self.ptr] = sink.into_sample().l;
            self.ptr += 1;
            if self.ptr >= self.local_data.len() {
                self.ptr = 0;
                self.shared_data.lock().unwrap().copy_from_slice(&self.local_data);
            }
            sink
        }
    }
}
