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
    let sdl_context = sdl2::init().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();

    let source = StaticSource::new("test85.wav");
    let ident: Ident<Sample> = Ident::new();
    //let sink = PrintSink::new();
    let sink = SDL2Sink::new(audio_subsystem);

    let p = pipe!(source, ident, sink);
    p.start();

    std::thread::sleep(std::time::Duration::from_millis(2000));
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
struct SDL2Sink {
    audio_subsystem: sdl2::AudioSubsystem,
    stop_handle: Option<Box<SDL2DeviceStopHandle>>
}
impl SDL2Sink {
    fn new(audio_subsystem: sdl2::AudioSubsystem) -> Self {
        Self {
            audio_subsystem: audio_subsystem,
            stop_handle: None
        }
    }
}
impl PullElement for SDL2Sink {
    type Sink = Sample;
    fn start<E>(&mut self, mut sink: E)
    where E: Element<Sink=(), Src=Self::Sink> + Send + Sync + 'static {
        let desired_spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(2),
            samples: None
        };
        let device = self.audio_subsystem.open_playback(None, &desired_spec, |_spec| {
            SDL2Callback { iter: move || {
                let sm = sink.next(()).to_float();
                match sm {
                    Sample::StereoF64 {l, r:_r} =>
                        return (l * ::std::i16::MAX as f64) as i16,
                    _ => panic!(),
                }
            }
        }
        }).unwrap();
        device.resume();
        self.stop_handle = Some(Box::new(device));
    }
    fn stop(&mut self) {
        match self.stop_handle {
            Some(ref handle) => handle.stop(),
            None => {}
        }
    }
}

struct SDL2Callback<F: FnMut() -> i16> {
    iter: F,
}
impl<F: FnMut() -> i16> AudioCallback for SDL2Callback<F>
where Self: Send + Sync {
    type Channel = i16;
    fn callback(&mut self, out: &mut [Self::Channel]) {
        for buf in out {
            *buf = (self.iter)();
        }
    }
}
trait SDL2DeviceStopHandle {
    fn stop(&self);
}
impl<CB: AudioCallback> SDL2DeviceStopHandle for AudioDevice<CB> {
    fn stop(&self) {
        self.pause();
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
trait PullElement {
    type Sink;
    fn start<E>(&mut self, sink: E)
        where E: Element<Sink=(), Src=Self::Sink> + Send + Sync + 'static;
    fn stop(&mut self);
}
trait Pipeline {
    fn start(self);
}
trait SinkPipeline {
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
impl<A, B> SinkPipeline for Pipe<A, B>
where A: Element<Sink=()> + Send + Sync + 'static,
      B: PullElement<Sink=A::Src> {
    fn start(mut self) {
        self.b.start(self.a);
        std::thread::sleep(std::time::Duration::from_millis(2000));
    }
}

