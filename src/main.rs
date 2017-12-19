extern crate rust_streamer;
use rust_streamer::*;
use rust_streamer::element::*;
use rust_streamer::wav::*;

fn main() {
    let source = WAVSource::new("test85.wav");
    //let source = FreqConv::new(source);

    let ident = Ident::new();

    //let tee = Tee::new(|x| {println!("{:?}", x)});

    let bc = FnElement::new(|x: WAVSample| {
        if let WAVSample::StereoI16 { l, r } = x {
            WAVSample::StereoI16 { l: l & !0xfff, r: r & !0xfff }
        } else {
            panic!()
        }
    });

    //let sink = PrintSink::new();
    let sink = CpalSink::new();

    let p = pipe!(source, bc, sink);

    p.start(&Context::new(48000));

    /*
    std::thread::spawn(move || {
    });
    std::thread::sleep(std::time::Duration::from_millis(20000));
    */
}

