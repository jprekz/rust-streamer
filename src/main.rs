extern crate cpal;

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
    //let sink = PrintSink::new();
    let sink = CpalSink::new();

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

struct CpalSink {
}
impl CpalSink {
    fn new() -> Self {
        CpalSink {}
    }
}
impl PullElement for CpalSink {
    type Sink = Sample;
    fn start<E>(&mut self, mut sink: E)
    where E: Element<Sink=(), Src=Self::Sink> + Send + Sync + 'static {
        use cpal::*;

        let endpoint = default_endpoint().expect("Failed to get default endpoint");
        let format = Format {
            channels: vec![ChannelPosition::FrontLeft, ChannelPosition::FrontRight],
            samples_rate: SamplesRate(44100),
            data_type: SampleFormat::F32
        };
        let event_loop = EventLoop::new();
        let voice_id = event_loop.build_voice(&endpoint, &format).unwrap();
        event_loop.play(voice_id);
        std::thread::spawn(move || {
            event_loop.run(move |_, buffer| {
                match buffer {
                    UnknownTypeBuffer::F32(mut buffer) => {
                        for sample in buffer.chunks_mut(format.channels.len()) {
                            let value = match sink.next(()).to_float() {
                                wav::Sample::StereoF64 {l, r:_r} =>
                                    l as f32,
                                _ => panic!(),
                            };
                            for out in sample.iter_mut() {
                                *out = value;
                            }
                        }
                    },
                    _ => panic!()
                };
            });
        });
    }
    fn stop(&mut self) {
        unimplemented!();
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

