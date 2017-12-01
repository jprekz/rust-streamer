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
    type Src = Format<Sample, AnyFreq>;
    fn next(&mut self, _sink: Self::Sink) -> Self::Src {
        self.pos += 1;
        Format {
            sample: self.wav.get_sample(self.pos - 1).unwrap(),
            freq: AnyFreq::new(self.wav.samplerate)
        }
    }
}

pub struct Ident<T> {t: ::std::marker::PhantomData<T>}
impl<T> Ident<T> {
    pub fn new() -> Self {
        Self { t: ::std::marker::PhantomData }
    }
}
impl<T> Element for Ident<T> {
    type Sink = T;
    type Src = T;
    fn next(&mut self, sink: Self::Sink) -> Self::Src {
        sink
    }
}

pub struct CpalSink {}
impl CpalSink {
    pub fn new() -> Self {
        CpalSink {}
    }
}
impl PullElement for CpalSink {
    type Sink = Format<Sample, F48000>;
    fn start<E>(&mut self, mut sink: E)
    where E: Element<Sink=(), Src=Self::Sink> + Send + Sync + 'static {
        use self::cpal::*;

        let endpoint = default_endpoint().expect("Failed to get default endpoint");
        let format = Format {
            channels: vec![ChannelPosition::FrontLeft, ChannelPosition::FrontRight],
            samples_rate: SamplesRate(48000),
            data_type: SampleFormat::F32
        };
        let event_loop = EventLoop::new();
        let voice_id = event_loop.build_voice(&endpoint, &format).unwrap();
        event_loop.play(voice_id);
        event_loop.run(move |_, buffer| {
            match buffer {
                UnknownTypeBuffer::F32(mut buffer) => {
                    for sample in buffer.chunks_mut(format.channels.len()) {
                        let values = match sink.next(()).sample.to_float() {
                            wav::Sample::StereoF64 {l, r} =>
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

pub struct PrintSink<T> {t: ::std::marker::PhantomData<T>}
impl<T> PrintSink<T> {
    pub fn new() -> Self {
        Self{ t: ::std::marker::PhantomData }
    }
}
impl<T> Element for PrintSink<T>
where T: std::fmt::Debug {
    type Sink = T;
    type Src = ();
    fn next(&mut self, sink: Self::Sink) -> Self::Src {
        println!("{:?}", sink);
    }
}

