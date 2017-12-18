extern crate rust_streamer;
use rust_streamer::*;
use rust_streamer::element::*;

fn main() {
    let source = StaticSource::new("test85.wav");
    //let source = FreqConv::new(source);

    let ident = Ident::new();

    let tee = Tee::new(|x| {println!("{:?}", x)});

    //let sink = PrintSink::new();
    let sink = CpalSink::new();

    let p = pipe!(source, ident, tee, sink);

    p.start(&());

    /*
    std::thread::spawn(move || {
        p.start(&());
    });
    std::thread::sleep(std::time::Duration::from_millis(20000));
    */
}

