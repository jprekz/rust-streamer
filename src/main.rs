extern crate rust_streamer;
use rust_streamer::*;
use rust_streamer::element::*;

fn main() {
    let source = WAVSource::new("test85.wav");
    //let source = FreqConv::new(source);

    let ident = Ident::new();

    //let tee = Tee::new(|x| {println!("{:?}", x)});

    //let sink = PrintSink::new();
    let sink = CpalSink::new();

    let p = pipe!(source, ident, sink);

    p.start(&Context::new(48000));

    /*
    std::thread::spawn(move || {
    });
    std::thread::sleep(std::time::Duration::from_millis(20000));
    */
}

