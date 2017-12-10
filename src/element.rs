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
impl Element for StaticSource {
    type Sink = ();
    type Src = WAVSample;
    type Freq = AnyFreq;
    fn next(&mut self, _sink: Self::Sink) -> Self::Src {
        self.pos += 1;
        self.wav.get_sample(self.pos - 1).unwrap()
    }
}

pub struct Ident<T, F> {
    t: ::std::marker::PhantomData<T>,
    f: ::std::marker::PhantomData<F>
}
impl<T,F> Ident<T, F> {
    pub fn new() -> Self {
        Self {
            t: ::std::marker::PhantomData,
            f: ::std::marker::PhantomData
        }
    }
}
impl<T, F: Freq> Element for Ident<T, F> {
    type Sink = T;
    type Src = T;
    type Freq = F;
    fn next(&mut self, sink: Self::Sink) -> Self::Src {
        sink
    }
}

pub struct CpalSink<F> {
    f: ::std::marker::PhantomData<F>
}
impl<F> CpalSink<F> {
    pub fn new() -> Self {
        CpalSink {
            f: ::std::marker::PhantomData
        }
    }
}
impl<F: ConstFreq> PullElement for CpalSink<F> {
    type Sink = WAVSample;
    type Freq = F;
    fn start<E>(&mut self, mut sink: E)
    where E: Element<Sink=(), Src=Self::Sink> + Send + Sync + 'static {
        use self::cpal::*;

        let endpoint = default_endpoint().expect("Failed to get default endpoint");
        let format = Format {
            channels: vec![ChannelPosition::FrontLeft, ChannelPosition::FrontRight],
            samples_rate: SamplesRate(F::F),
            data_type: SampleFormat::F32
        };
        let event_loop = EventLoop::new();
        let voice_id = event_loop.build_voice(&endpoint, &format).unwrap();
        event_loop.play(voice_id);
        event_loop.run(move |_, buffer| {
            match buffer {
                UnknownTypeBuffer::F32(mut buffer) => {
                    for sample in buffer.chunks_mut(format.channels.len()) {
                        let values = match sink.next(()).to_float() {
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

pub struct PrintSink<T, F> {
    t: ::std::marker::PhantomData<T>,
    f: ::std::marker::PhantomData<F>
}
impl<T, F> PrintSink<T, F> {
    pub fn new() -> Self {
        Self {
            t: ::std::marker::PhantomData,
            f: ::std::marker::PhantomData
        }
    }
}
impl<T, F: Freq> Element for PrintSink<T, F>
where T: std::fmt::Debug {
    type Sink = T;
    type Src = ();
    type Freq = F;
    fn next(&mut self, sink: Self::Sink) -> Self::Src {
        println!("{:?}", sink);
    }
}

