extern crate cpal;
extern crate rand;

use super::*;
use super::wav::*;
use super::sample::*;
use super::dsp::*;

use std::fs::File;
use std::marker::PhantomData;

// Input / Output

pub struct WAVSource<Src> {
    wav: WAV,
    pos: usize,
    src_type: PhantomData<Src>,
}
impl<Src> WAVSource<Src> {
    pub fn new(filename: &str) -> Self {
        let file = File::open(filename).unwrap();
        let wav = WAV::new(file);
        Self {
            wav: wav,
            pos: 0,
            src_type: PhantomData,
        }
    }
}
impl<Ctx, Src> Element<(), Ctx> for WAVSource<Src>
where
    Src: Sample,
    Src::Member: FromSampleType<i16>,
{
    type Src = Src;
    fn next(&mut self, _sink: (), _ctx: &Ctx) -> Src {
        self.pos += 1;
        match self.wav.get_sample_as::<Src>(self.pos - 1) {
            Some(s) => s,
            None => {
                self.pos = 1;
                self.wav.get_sample_as::<Src>(self.pos - 1).unwrap()
            }
        }
    }
}

pub struct SineWave<Src> {
    freq: f64,
    pos: usize,
    src_type: PhantomData<Src>,
}
impl<Src> SineWave<Src> {
    pub fn new(freq: f64) -> Self {
        Self {
            freq: freq,
            pos: 0,
            src_type: PhantomData,
        }
    }
}
impl<Ctx, Src> Element<(), Ctx> for SineWave<Src>
where
    Ctx: FreqCtx,
    Src: FromSample<Mono<f64>>,
{
    type Src = Src;
    fn next(&mut self, _sink: (), ctx: &Ctx) -> Src {
        self.pos += 1;
        Mono::new((2.0 * std::f64::consts::PI * self.pos as f64 * self.freq / ctx.get_freq() as f64).sin())
            .into_sample()
    }
}

pub struct WhiteNoise<Src> {
    src_type: PhantomData<Src>,
}
impl<Src> WhiteNoise<Src> {
    pub fn new() -> Self {
        Self {
            src_type: PhantomData,
        }
    }
}
impl<Ctx, Src> Element<(), Ctx> for WhiteNoise<Src>
where
    Src: FromSample<Mono<f64>>,
{
    type Src = Src;
    fn next(&mut self, _sink: (), _ctx: &Ctx) -> Src {
        use self::rand::distributions::{IndependentSample, Range};
        Mono::new(Range::new(-1f64, 1.).ind_sample(&mut rand::thread_rng())).into_sample()
    }
}

pub struct DefaultSink;
impl DefaultSink {
    pub fn new() -> Self {
        DefaultSink {}
    }
}
impl<S, Ctx> PullElement<S, Ctx> for DefaultSink
where
    S: IntoSample<Stereo<f32>>,
    Ctx: FreqCtx + Sync,
{
    fn start<E>(&mut self, mut sink: E, ctx: &Ctx)
    where
        E: Element<(), Ctx, Src = S> + Send,
    {
        use self::cpal::*;

        let endpoint = default_endpoint().expect("Failed to get default endpoint");
        let format = Format {
            channels: vec![ChannelPosition::FrontLeft, ChannelPosition::FrontRight],
            samples_rate: SamplesRate(ctx.get_freq()),
            data_type: SampleFormat::F32,
        };
        let event_loop = EventLoop::new();
        let voice_id = event_loop
            .build_voice(&endpoint, &format)
            .expect("Failed to build voice");
        event_loop.play(voice_id);
        event_loop.run(move |_, buffer| {
            match buffer {
                UnknownTypeBuffer::F32(mut buffer) => {
                    for sample in buffer.chunks_mut(format.channels.len()) {
                        let Stereo { l, r } = sink.next((), ctx).into_sample();
                        sample[0] = l;
                        sample[1] = r;
                    }
                }
                _ => panic!(),
            };
        });
    }
    fn stop(&mut self) {
        unimplemented!();
    }
}

pub struct NullSink;
impl NullSink {
    pub fn new() -> Self {
        Self {}
    }
}
impl<T, Ctx> Element<T, Ctx> for NullSink {
    type Src = ();
    fn next(&mut self, _sink: T, _ctx: &Ctx) {
        // do nothing
    }
}

pub struct PrintSink;
impl PrintSink {
    pub fn new() -> Self {
        Self {}
    }
}
impl<T, Ctx> Element<T, Ctx> for PrintSink
where
    T: std::fmt::Debug,
{
    type Src = ();
    fn next(&mut self, sink: T, _ctx: &Ctx) {
        println!("{:?}", sink);
    }
}

// Common Element

pub struct Ident;
impl Ident {
    pub fn new() -> Self {
        Self {}
    }
}
impl<T, Ctx> Element<T, Ctx> for Ident {
    type Src = T;
    fn next(&mut self, sink: T, _ctx: &Ctx) -> T {
        sink
    }
}

pub struct Tee<F> {
    f: F,
}
impl<F> Tee<F> {
    pub fn new(f: F) -> Self {
        Tee { f: f }
    }
}
impl<T, Ctx, F> Element<T, Ctx> for Tee<F>
where
    F: FnMut(T),
    T: Copy,
{
    type Src = T;
    fn next(&mut self, sink: T, _ctx: &Ctx) -> T {
        (self.f)(sink);
        sink
    }
}

pub struct FnElement<F> {
    f: F,
}
impl<F> FnElement<F> {
    pub fn new(f: F) -> Self {
        FnElement { f: f }
    }
}
impl<T1, T2, Ctx, F> Element<T1, Ctx> for FnElement<F>
where
    F: FnMut(T1) -> T2,
    T1: Copy,
    T2: Copy,
{
    type Src = T2;
    fn next(&mut self, sink: T1, _ctx: &Ctx) -> T2 {
        (self.f)(sink)
    }
}

// DSP Element

pub struct LowPassFilter {
    freq: f64,
    q: f64,
    iir_l: BiQuadIIR,
    iir_r: BiQuadIIR,
}
impl LowPassFilter {
    pub fn new(freq: f64, q: f64) -> Self {
        Self {
            freq: freq,
            q: q,
            iir_l: Default::default(),
            iir_r: Default::default(),
        }
    }
}
impl<T, Ctx> Element<T, Ctx> for LowPassFilter
where
    Ctx: FreqCtx,
    T: IntoSample<Stereo<f64>> + FromSample<Stereo<f64>>,
{
    type Src = T;
    fn init_with_ctx(&mut self, ctx: &Ctx) {
        self.iir_l = BiQuadIIR::new_low_pass_filter(ctx.get_freq() as f64, self.freq, self.q);
        self.iir_r = BiQuadIIR::new_low_pass_filter(ctx.get_freq() as f64, self.freq, self.q);
    }
    fn next(&mut self, sink: T, _ctx: &Ctx) -> T {
        let sink = sink.into_sample();
        let src = Stereo {
            l: self.iir_l.next(sink.l),
            r: self.iir_r.next(sink.r),
        };
        src.into_sample()
    }
}

pub struct HighPassFilter {
    freq: f64,
    q: f64,
    iir_l: BiQuadIIR,
    iir_r: BiQuadIIR,
}
impl HighPassFilter {
    pub fn new(freq: f64, q: f64) -> Self {
        Self {
            freq: freq,
            q: q,
            iir_l: Default::default(),
            iir_r: Default::default(),
        }
    }
}
impl<T, Ctx> Element<T, Ctx> for HighPassFilter
where
    Ctx: FreqCtx,
    T: IntoSample<Stereo<f64>> + FromSample<Stereo<f64>>,
{
    type Src = T;
    fn init_with_ctx(&mut self, ctx: &Ctx) {
        self.iir_l = BiQuadIIR::new_high_pass_filter(ctx.get_freq() as f64, self.freq, self.q);
        self.iir_r = BiQuadIIR::new_high_pass_filter(ctx.get_freq() as f64, self.freq, self.q);
    }
    fn next(&mut self, sink: T, _ctx: &Ctx) -> T {
        let sink = sink.into_sample();
        let src = Stereo {
            l: self.iir_l.next(sink.l),
            r: self.iir_r.next(sink.r),
        };
        src.into_sample()
    }
}

pub struct BandPassFilter {
    freq: f64,
    bw: f64,
    iir_l: BiQuadIIR,
    iir_r: BiQuadIIR,
}
impl BandPassFilter {
    pub fn new(freq: f64, bw: f64) -> Self {
        Self {
            freq: freq,
            bw: bw,
            iir_l: Default::default(),
            iir_r: Default::default(),
        }
    }
}
impl<T, Ctx> Element<T, Ctx> for BandPassFilter
where
    Ctx: FreqCtx,
    T: IntoSample<Stereo<f64>> + FromSample<Stereo<f64>>,
{
    type Src = T;
    fn init_with_ctx(&mut self, ctx: &Ctx) {
        self.iir_l = BiQuadIIR::new_band_pass_filter(ctx.get_freq() as f64, self.freq, self.bw);
        self.iir_r = BiQuadIIR::new_band_pass_filter(ctx.get_freq() as f64, self.freq, self.bw);
    }
    fn next(&mut self, sink: T, _ctx: &Ctx) -> T {
        let sink = sink.into_sample();
        let src = Stereo {
            l: self.iir_l.next(sink.l),
            r: self.iir_r.next(sink.r),
        };
        src.into_sample()
    }
}

pub struct NotchFilter {
    freq: f64,
    bw: f64,
    iir_l: BiQuadIIR,
    iir_r: BiQuadIIR,
}
impl NotchFilter {
    pub fn new(freq: f64, bw: f64) -> Self {
        Self {
            freq: freq,
            bw: bw,
            iir_l: Default::default(),
            iir_r: Default::default(),
        }
    }
}
impl<T, Ctx> Element<T, Ctx> for NotchFilter
where
    Ctx: FreqCtx,
    T: IntoSample<Stereo<f64>> + FromSample<Stereo<f64>>,
{
    type Src = T;
    fn init_with_ctx(&mut self, ctx: &Ctx) {
        self.iir_l = BiQuadIIR::new_notch_filter(ctx.get_freq() as f64, self.freq, self.bw);
        self.iir_r = BiQuadIIR::new_notch_filter(ctx.get_freq() as f64, self.freq, self.bw);
    }
    fn next(&mut self, sink: T, _ctx: &Ctx) -> T {
        let sink = sink.into_sample();
        let src = Stereo {
            l: self.iir_l.next(sink.l),
            r: self.iir_r.next(sink.r),
        };
        src.into_sample()
    }
}

pub struct LowShelfFilter {
    freq: f64,
    q: f64,
    gain: f64,
    iir_l: BiQuadIIR,
    iir_r: BiQuadIIR,
}
impl LowShelfFilter {
    pub fn new(freq: f64, q: f64, gain: f64) -> Self {
        Self {
            freq: freq,
            q: q,
            gain: gain,
            iir_l: Default::default(),
            iir_r: Default::default(),
        }
    }
}
impl<T, Ctx> Element<T, Ctx> for LowShelfFilter
where
    Ctx: FreqCtx,
    T: IntoSample<Stereo<f64>> + FromSample<Stereo<f64>>,
{
    type Src = T;
    fn init_with_ctx(&mut self, ctx: &Ctx) {
        self.iir_l = BiQuadIIR::new_low_shelf_filter(ctx.get_freq() as f64, self.freq, self.q, self.gain);
        self.iir_r = BiQuadIIR::new_low_shelf_filter(ctx.get_freq() as f64, self.freq, self.q, self.gain);
    }
    fn next(&mut self, sink: T, _ctx: &Ctx) -> T {
        let sink = sink.into_sample();
        let src = Stereo {
            l: self.iir_l.next(sink.l),
            r: self.iir_r.next(sink.r),
        };
        src.into_sample()
    }
}

pub struct HighShelfFilter {
    freq: f64,
    q: f64,
    gain: f64,
    iir_l: BiQuadIIR,
    iir_r: BiQuadIIR,
}
impl HighShelfFilter {
    pub fn new(freq: f64, q: f64, gain: f64) -> Self {
        Self {
            freq: freq,
            q: q,
            gain: gain,
            iir_l: Default::default(),
            iir_r: Default::default(),
        }
    }
}
impl<T, Ctx> Element<T, Ctx> for HighShelfFilter
where
    Ctx: FreqCtx,
    T: IntoSample<Stereo<f64>> + FromSample<Stereo<f64>>,
{
    type Src = T;
    fn init_with_ctx(&mut self, ctx: &Ctx) {
        self.iir_l = BiQuadIIR::new_high_shelf_filter(ctx.get_freq() as f64, self.freq, self.q, self.gain);
        self.iir_r = BiQuadIIR::new_high_shelf_filter(ctx.get_freq() as f64, self.freq, self.q, self.gain);
    }
    fn next(&mut self, sink: T, _ctx: &Ctx) -> T {
        let sink = sink.into_sample();
        let src = Stereo {
            l: self.iir_l.next(sink.l),
            r: self.iir_r.next(sink.r),
        };
        src.into_sample()
    }
}

pub struct PeakingFilter {
    freq: f64,
    bw: f64,
    gain: f64,
    iir_l: BiQuadIIR,
    iir_r: BiQuadIIR,
}
impl PeakingFilter {
    pub fn new(freq: f64, bw: f64, gain: f64) -> Self {
        Self {
            freq: freq,
            bw: bw,
            gain: gain,
            iir_l: Default::default(),
            iir_r: Default::default(),
        }
    }
}
impl<T, Ctx> Element<T, Ctx> for PeakingFilter
where
    Ctx: FreqCtx,
    T: IntoSample<Stereo<f64>> + FromSample<Stereo<f64>>,
{
    type Src = T;
    fn init_with_ctx(&mut self, ctx: &Ctx) {
        self.iir_l = BiQuadIIR::new_peaking_filter(ctx.get_freq() as f64, self.freq, self.bw, self.gain);
        self.iir_r = BiQuadIIR::new_peaking_filter(ctx.get_freq() as f64, self.freq, self.bw, self.gain);
    }
    fn next(&mut self, sink: T, _ctx: &Ctx) -> T {
        let sink = sink.into_sample();
        let src = Stereo {
            l: self.iir_l.next(sink.l),
            r: self.iir_r.next(sink.r),
        };
        src.into_sample()
    }
}

pub struct AllPassFilter {
    freq: f64,
    q: f64,
    iir_l: BiQuadIIR,
    iir_r: BiQuadIIR,
}
impl AllPassFilter {
    pub fn new(freq: f64, q: f64) -> Self {
        Self {
            freq: freq,
            q: q,
            iir_l: Default::default(),
            iir_r: Default::default(),
        }
    }
}
impl<T, Ctx> Element<T, Ctx> for AllPassFilter
where
    Ctx: FreqCtx,
    T: IntoSample<Stereo<f64>> + FromSample<Stereo<f64>>,
{
    type Src = T;
    fn init_with_ctx(&mut self, ctx: &Ctx) {
        self.iir_l = BiQuadIIR::new_all_pass_filter(ctx.get_freq() as f64, self.freq, self.q);
        self.iir_r = BiQuadIIR::new_all_pass_filter(ctx.get_freq() as f64, self.freq, self.q);
    }
    fn next(&mut self, sink: T, _ctx: &Ctx) -> T {
        let sink = sink.into_sample();
        let src = Stereo {
            l: self.iir_l.next(sink.l),
            r: self.iir_r.next(sink.r),
        };
        src.into_sample()
    }
}
