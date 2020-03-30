use rust_streamer::context::DefaultContext;
use rust_streamer::element::*;
use rust_streamer::graphic::*;
use rust_streamer::sample::*;
use rust_streamer::*;

fn main() {
    let p = pipe!(
        WAVSource::<Stereo<f64>>::new("examples/test85.wav").unwrap(),
        Spectrum::new(1024),
        LowPassFilter::new(1000.0, 1.0 / 2f64.sqrt()),
        Spectrum::new(1024),
        DefaultSink::new()
    );

    p.start(DefaultContext::new());
}
