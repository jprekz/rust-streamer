extern crate rust_streamer;
use rust_streamer::*;
use rust_streamer::element::*;

fn main() {
    let source = StaticSource::new("test85.wav");
    let source: FreqConv<_, F48000> = FreqConv::new(source);

    let ident = Ident::new();

    //let sink = PrintSink::new();
    let sink = CpalSink::new();

    let p = pipe!(source, ident, sink);

    std::thread::spawn(move || {
        p.start();
    });

    std::thread::sleep(std::time::Duration::from_millis(20000));
}

