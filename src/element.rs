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
      Ctx: FreqCtx {
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
where F: Fn(T),
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
where F: Fn(T1) -> T2,
      T1: Copy,
      T2: Copy {
    type Src = T2;
    fn next(&mut self, sink: T1, _ctx: &Ctx) -> T2 {
        (self.f)(sink)
    }
}
