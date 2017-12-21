extern crate rust_streamer;
use rust_streamer::*;
use rust_streamer::element::*;
use rust_streamer::sample::*;

fn main() {
    let source = WAVSource::<Stereo<f64>>::new("test85.wav");
    //let source = FreqConv::new(source);

    let ident = Ident::new();


    let bc = FnElement::new(|x: Stereo<f64>| {
        let x: Stereo<i16> = x.into_sample();
        let x: Stereo<i16> = x.map(|s: i16| { s & !0x3fff });
        x
    });

    let tee = Tee::new(|x| {println!("{:?}", x)});

    let sink = PrintSink::new();
    //let sink = CpalSink::new();
    //let sink = NullSink::new();

    let p = pipe!(source, bc, tee, sink);

    p.start(&Context::new(48000));

    /*
    std::thread::spawn(move || {
    });
    std::thread::sleep(std::time::Duration::from_millis(20000));
    */
}

