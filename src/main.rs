extern crate rust_streamer;
use rust_streamer::*;
use rust_streamer::element::*;
use rust_streamer::wav::*;
use rust_streamer::sample::*;

fn main() {
    let source = WAVSource::<Stereo<f64>>::new("test85.wav");
    //let source = FreqConv::new(source);

    let ident = Ident::new();

    //let tee = Tee::new(|x| {println!("{:?}", x)});

    /*
    let bc = FnElement::new(|x: WAVSample| {
        if let WAVSample::StereoI16 { l, r } = x {
            WAVSample::StereoI16 { l: l & !0x3fff, r: r & !0x3fff }
        } else {
            panic!()
        }
    });
    */

    let sink = PrintSink::new();
    //let sink = CpalSink::new();
    //let sink = NullSink::new();

    let p = pipe!(source, sink);

    p.start(&Context::new(48000));

    /*
    std::thread::spawn(move || {
    });
    std::thread::sleep(std::time::Duration::from_millis(20000));
    */
}

