use rust_streamer::*;
use rust_streamer::element::*;
use rust_streamer::sample::*;

fn main() {
    let source = SineWave::<Stereo<f64>>::new(440.0);
    let sink = DefaultSink::new();

    let p = pipe!(source, sink);

    p.start(&Context::new(44100));
}
