extern crate cpal;

use super::*;
use super::wav::*;

use std::fs::File;

pub struct StaticSource {
    wav: WAV,
    pos: usize,
}
impl StaticSource {
    pub fn new(filename: &str) -> Self {
        let file = File::open(filename).unwrap();
        let wav = WAV::new(file);
        Self {
            wav: wav,
            pos: 0,
        }
    }
}
impl<Ctx> Element<(), Ctx> for StaticSource {
    type Src = WAVSample;
    fn next(&mut self, _sink: (), _ctx: &Ctx) -> WAVSample {
        self.pos += 1;
        self.wav.get_sample(self.pos - 1).unwrap()
    }
}

pub struct Ident;
impl Ident {
    pub fn new() -> Self { Self{} }
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
where F: Fn(T),
      T: Copy {
    type Src = T;
    fn next(&mut self, sink: T, _ctx: &Ctx) -> T {
        (self.f)(sink);
        sink
    }
}

pub struct CpalSink {
}
impl CpalSink {
    pub fn new() -> Self {
        CpalSink {
        }
    }
}
impl<Ctx> PullElement<WAVSample, Ctx> for CpalSink
where Ctx: FreqCtx {
    fn start<E>(&mut self, mut sink: E, ctx: &Ctx)
    where E: Element<(), Ctx, Src=WAVSample> + Send + Sync + 'static {
        use self::cpal::*;

        let endpoint = default_endpoint().expect("Failed to get default endpoint");
        let format = Format {
            channels: vec![ChannelPosition::FrontLeft, ChannelPosition::FrontRight],
            samples_rate: SamplesRate(ctx.get_freq()),
            data_type: SampleFormat::F32
        };
        let event_loop = EventLoop::new();
        let voice_id = event_loop.build_voice(&endpoint, &format).unwrap();
        event_loop.play(voice_id);
        event_loop.run(move |_, buffer| {
            match buffer {
                UnknownTypeBuffer::F32(mut buffer) => {
                    for sample in buffer.chunks_mut(format.channels.len()) {
                        let values = match sink.next((), ctx).to_float() {
                            wav::WAVSample::StereoF64 {l, r} =>
                                [l as f32, r as f32],
                            _ => panic!(),
                        };
                        sample[0] = values[0];
                        sample[1] = values[1];
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

pub struct PrintSink<T> {
    t: ::std::marker::PhantomData<T>,
}
impl<T> PrintSink<T> {
    pub fn new() -> Self {
        Self {
            t: ::std::marker::PhantomData,
        }
    }
}
impl<T, Ctx> Element<T, Ctx> for PrintSink<T>
where T: std::fmt::Debug {
    type Src = ();
    fn next(&mut self, sink: T, _ctx: &Ctx) {
        println!("{:?}", sink);
    }
}

