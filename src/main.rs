extern crate rust_streamer;
use rust_streamer::*;
use rust_streamer::element::*;
use rust_streamer::sample::*;
use rust_streamer::graphic::*;

fn main() {
    let source: WAVSource<Stereo<f64>> = WAVSource::new("test85.wav");

    let bc = FnElement::new(|x: Stereo<i16>| {
        x.map(|s: i16| { s & !0x1fff })
    });

    //let tee = Tee::new(|x| {println!("{:?}", x)});

    //let sink = PrintSink::new();
    let sink = CpalSink::new();

    let p = pipe!(
        source,
        Spectrum::new(1024),
        PeakingFilter::new(1000.0, 2.0, -30.0),
        Spectrum::new(1024),
        Oscillo::new(640),
        sink
    );

    p.start(&Context::new(44100));
}
