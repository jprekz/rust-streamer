extern crate rust_streamer;
use rust_streamer::*;
use rust_streamer::element::*;
use rust_streamer::sample::*;

fn main() {
    let source = WAVSource::new("test85.wav");
    //let source = FreqConv::new(source);

    let bc = FnElement::new(|x: Stereo<i16>| {
        x.map(|s: i16| { s & !0x3fff })
    });

    //let tee = Tee::new(|x| {println!("{:?}", x)});

    //let sink = PrintSink::new();
    let sink = CpalSink::new();

    let p = pipe!(source, bc, sink);

    p.start(&Context::new(48000));
}
