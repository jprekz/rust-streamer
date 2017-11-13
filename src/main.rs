extern crate sdl2;

use std::fs::File;

mod wav;
use wav::*;

macro_rules! pipe {
    ( $e1:expr, $e2:expr ) => {
        Pipe::new($e1, $e2);
    };
    ( $e1:expr, $e2:expr, $( $e:expr ),* ) => {{
        let p = Pipe::new($e1, $e2);
        $(
            let p = Pipe::new(p, $e);
        )*
        p
    }}
}

fn main() {
    let source = StaticSource::new("test85.wav");
    let ident: Ident<Sample> = Ident::new();
    let ident2: Ident<Sample> = Ident::new();
    let sink = PrintSink::new();

    let p = pipe!(source, ident, ident2, sink);

    p.start();
}

fn _test() {
    let sdl_context = sdl2::init().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();

    let mut ss = StaticSource::new("test85.wav");
    let mut s = SDL2Sink::new(audio_subsystem, move || {
        let sm = ss.next(()).to_float();
        match sm {
            Sample::StereoF64 {l, r:_r} =>
                return (l * ::std::i16::MAX as f64) as i16,
            _ => panic!(),
        }
    });

    s.start();
}

struct StaticSource {
    wav: WAV,
    pos: usize,
}
impl StaticSource {
    fn new(filename: &str) -> Self {
        let file = File::open(filename).unwrap();
        let wav = WAV::new(file);
        Self {
            wav: wav,
            pos: 0,
        }
    }
}
impl Element for StaticSource {
    type Sink = ();
    type Src = Sample;
    fn next(&mut self, _sink: Self::Sink) -> Self::Src {
        self.pos += 1;
        self.wav.get_sample(self.pos - 1).unwrap()
    }
}

struct Ident<T> {t: std::marker::PhantomData<T>}
impl<T> Ident<T> {
    fn new() -> Self {
        Self { t: std::marker::PhantomData }
    }
}
impl<T> Element for Ident<T> {
    type Sink = T;
    type Src = T;
    fn next(&mut self, sink: Self::Sink) -> Self::Src {
        sink
    }
}

use self::sdl2::audio::*;
struct SDL2Sink<F: FnMut() -> i16> where F: Send + Sync {
    device: AudioDevice<SDL2Callback<F>>
}
impl<F: FnMut() -> i16> SDL2Sink<F> where F: Send + Sync {
    fn new(audio_subsystem: sdl2::AudioSubsystem, f: F) -> Self {
        let desired_spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(2),
            samples: None
        };
        let device = audio_subsystem.open_playback(None, &desired_spec, |_spec| {
            SDL2Callback {iter: f}
        }).unwrap();
        Self {
            device: device
        }
    }
    fn start(&mut self) {
        self.device.resume();
    }
}
struct SDL2Callback<F: FnMut() -> i16> {
    iter: F,
}
impl<F: FnMut() -> i16> AudioCallback for SDL2Callback<F>
where SDL2Callback<F>: Send + Sync,
      F: Send + Sync {
    type Channel = i16;
    fn callback(&mut self, out: &mut [Self::Channel]) {
        for buf in out {
            *buf = (self.iter)();
        }
    }
}

struct PrintSink {}
impl PrintSink {
    fn new() -> Self {
        Self{}
    }
}
impl Element for PrintSink {
    type Sink = Sample;
    type Src = ();
    fn next(&mut self, sink: Self::Sink) -> Self::Src {
        println!("{:?}", sink);
    }
}

// Core

trait Element {
    type Sink;
    type Src;
    fn next(&mut self, sink: Self::Sink) -> Self::Src;
}
trait Pipeline {
    fn start(self);
}

struct Pipe<A, B> {
    a: A,
    b: B,
}
impl<A, B> Pipe<A, B> {
    fn new(a: A, b: B) -> Self {
        Self {a: a, b: b}
    }
}
impl<A, B> Element for Pipe<A, B>
where A: Element,
      B: Element,
      A::Src: Into<B::Sink> {
    type Sink = A::Sink;
    type Src = B::Src;
    fn next(&mut self, sink: Self::Sink) -> Self::Src {
        self.b.next(self.a.next(sink).into())
    }
}
impl<A, B> Pipeline for Pipe<A, B>
where Self: Element<Sink=(), Src=()> {
    fn start(mut self) {
        loop {
            self.next(());
        }
    }
}

