extern crate rust_streamer;
use rust_streamer::*;
use rust_streamer::element::*;
use rust_streamer::sample::*;

fn main() {
    let source: WAVSource<Stereo<i16>> = WAVSource::new("test85.wav");
    //let source = FreqConv::new(source);

    let bc = FnElement::new(|x: Stereo<i16>| {
        x.map(|s: i16| { s & !0x3fff })
    });

    let mut b = [Stereo { l: 0, r: 0 }; 8];
    let lp = FnElement::new(move |x: Stereo<i16>| {
        b.push(x);
        Stereo::<i16> {
            l: b.iter().map(|s| { s.l / b.len() as i16 }).sum(),
            r: b.iter().map(|s| { s.r / b.len() as i16 }).sum()
        }
    });

    //let tee = Tee::new(|x| {println!("{:?}", x)});

    //let sink = PrintSink::new();
    let sink = CpalSink::new();

    let p = pipe!(source, lp, sink);

    p.start(&Context::new(48000));
}
