extern crate sdl2;

use std::fs::File;

mod wav;
use wav::*;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();

    let source = StaticSource::new();
    let ident: Ident<Sample> = Ident::new();
    let sink = SDL2Sink::new(audio_subsystem);

}

struct StaticSource {
    wav: WAV,
    pos: usize,
}
impl StaticSource {
    fn new() -> Self {
        let file = File::open("test85.wav").unwrap();
        let wav = WAV::new(file);
        Self {
            wav: wav,
            pos: 0,
        }
    }
}
impl Element for StaticSource {
    type Sink = ();
    type Src = Sample;
    fn next(&mut self, _sink: Self::Sink) -> Self::Src {
        self.pos += 1;
        self.wav.get_sample(self.pos - 1).unwrap()
    }
    fn settings() -> Settings {
        Settings {
        }
    }
}

struct Ident<T> {t: std::marker::PhantomData<T>}
impl<T> Ident<T> {
    fn new() -> Self {
        Self { t: std::marker::PhantomData }
    }
}
impl<T> Element for Ident<T> {
    type Sink = T;
    type Src = T;
    fn next(&mut self, sink: Self::Sink) -> Self::Src {
        sink
    }
    fn settings() -> Settings {
        Settings {
        }
    }
}

use self::sdl2::audio::*;
struct SDL2Sink {
    device: AudioDevice<SDL2Callback>
}
impl SDL2Sink {
    fn new(audio_subsystem: sdl2::AudioSubsystem) -> Self {
        let desired_spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(2),
            samples: None
        };
        let device = audio_subsystem.open_playback(None, &desired_spec, |_spec| {
            SDL2Callback {}
        }).unwrap();
        Self {
            device: device
        }
    }
    fn start(&mut self) {
        self.device.resume();
    }
}
impl Element for SDL2Sink {
    type Sink = Sample;
    type Src =();
    fn next(&mut self, _sink: Self::Sink) -> Self::Src {
    }
    fn settings() -> Settings {
        Settings {
        }
    }
}

struct SDL2Callback {  }
impl AudioCallback for SDL2Callback
where SDL2Callback: Send + Sync {
    type Channel = i16;
    fn callback(&mut self, out: &mut [Self::Channel]) {
        for buf in out {
            *buf = 0i16;
        }
    }
}

struct Settings {
}
trait Element {
    type Sink;
    type Src;
    fn next(&mut self, sink: Self::Sink) -> Self::Src;
    fn settings() -> Settings;
}

