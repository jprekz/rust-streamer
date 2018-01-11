extern crate rust_streamer;
use rust_streamer::*;
use rust_streamer::element::*;
use rust_streamer::sample::*;
use rust_streamer::graphic::*;

fn main() {
    let source = WAVSource::<Stereo<f64>>::new("test85.wav");
    let noise = WhiteNoise::<Stereo<f64>>::new();
    let gain = Gain::new(-10.0);
    let limiter = Limiter::new(0.0);
    let spectrum = Spectrum::new(1024);
    let oscillo = Oscillo::new(640);
    let sink = DefaultSink::new();

    let p = pipe!(
        fork!(
            source,
            pipe!(noise, gain)
        ),
        limiter,
        spectrum,
        oscillo,
        sink
    );

    p.start(&Context::new(44100));
/*

    let source: WAVSource<Stereo<f64>> = WAVSource::new("test85.wav");

    let bc = FnElement::new(|x: Stereo<i16>| {
        x.map(|s: i16| { s & !0x1fff })
    });

    //let tee = Tee::new(|x| {println!("{:?}", x)});

    //let sink = PrintSink::new();
    let sink = DefaultSink::new();

    let p = pipe!(
        source,
        Spectrum::new(1024),
        PeakingFilter::new(1000.0, 2.0, -30.0),
        Spectrum::new(1024),
        Oscillo::new(640),
        sink
    );

    p.start(&Context::new(44100));*/
}
