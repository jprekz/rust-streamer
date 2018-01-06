extern crate rust_streamer;
use rust_streamer::*;
use rust_streamer::element::*;
use rust_streamer::sample::*;
use rust_streamer::graphic::*;

fn main() {
    let source: WAVSource<Stereo<f64>> = WAVSource::new("test85.wav");
    //let source = FreqConv::new(source);

    let osc: SineWave<Stereo<f64>> = SineWave::new(440.0);

    let bc = FnElement::new(|x: Stereo<i16>| {
        x.map(|s: i16| { s & !0x1fff })
    });

    //let oscillo = Oscillo::new(640);
    let spectrum = Spectrum::new(1024*2);
    //let oscillo2 = Oscillo::new(640);
    let spectrum2 = Spectrum::new(1024*2);

    //let tee = Tee::new(|x| {println!("{:?}", x)});

    //let sink = PrintSink::new();
    let sink = CpalSink::new();

    let p = pipe!(source, spectrum, LowPassFilter::new(440.0, 2.0), spectrum2, sink);

    p.start(&Context::new(44100));
}
