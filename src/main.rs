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

    let mut b = [Stereo { l: 0, r: 0 }; 4];
    let lp = FnElement::new(move |x: Stereo<i16>| {
        b[0] = b[1];
        b[1] = b[2];
        b[2] = b[3];
        b[3] = x;
        Stereo {
            l: b[0].l/4 + b[1].l/4 + b[2].l/4 + b[3].l/4,
            r: b[0].r/4 + b[1].r/4 + b[2].r/4 + b[3].r/4
        }
    });

    //let tee = Tee::new(|x| {println!("{:?}", x)});

    let mut buf = vec![0i16; 2048];
    let mut ptr = 0usize;
    let analyze = Tee::new(move |x: Stereo<i16>| {
        buf[ptr] = x.l;
        ptr += 1;
        if ptr >= 2048 {
            ptr = 0;
            println!("{:?}", buf);
        }
    });

    //let sink = PrintSink::new();
    let sink = CpalSink::new();

    let p = pipe!(source, lp, sink);

    p.start(&Context::new(48000));
}
