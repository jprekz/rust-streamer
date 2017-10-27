extern crate sdl2;

use std::fs::File;

mod wav;
use wav::*;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();

    let source = StaticSource::new();
    let ident: Ident<Sample> = Ident::new();
    let sink = PrintSink::new();

    let p = Pipe::new(source, ident);
    let mut p = Pipe::new(p, sink);

    let mut ss = StaticSource::new();
    let mut s = SDL2Sink::new(audio_subsystem, move || {
        let sm = ss.next(()).to_float();
        match sm {
            Sample::StereoF64 {l, r:_r} =>
                return (l * ::std::i16::MAX as f64) as i16,
            _ => panic!(),
        }
    });
    s.start();

    loop {
        p.next(());
    }
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
    fn settings() -> Settings {
        Settings {
        }
    }
}

struct StaticSource {
    wav: WAV,
    pos: usize,
}
impl StaticSource {
    fn new() -> Self {
        let file = File::open("test85.wav").unwrap();
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
    fn settings() -> Settings {
        Settings {
        }
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
    fn settings() -> Settings {
        Settings {
        }
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
    fn settings() -> Settings {
        Settings {
        }
    }
}


struct Settings {
}
trait Element {
    type Sink;
    type Src;
    fn next(&mut self, sink: Self::Sink) -> Self::Src;
    fn settings() -> Settings;
}

