pub mod element;

mod wav;

// core traits

pub trait Element {
    type Sink;
    type Src;
    type Freq: Freq;
    fn next(&mut self, sink: Self::Sink) -> Self::Src;
}
pub trait PullElement {
    type Sink;
    fn start<E>(&mut self, sink: E)
        where E: Element<Sink=(), Src=Self::Sink> + Send + Sync + 'static;
    fn stop(&mut self);
}
pub trait PushElement {
    type Src;
    fn start<E>(&mut self, src: E)
        where E: Element<Sink=Self::Src, Src=()> + Send + Sync + 'static;
    fn stop(&mut self);
}
pub trait Pipeline {
    fn start(self);
}
pub trait SinkPipeline {
    fn start(self);
}

// pipe

pub struct Pipe<A, B> {
    a: A,
    b: B,
}
impl<A, B> Pipe<A, B> {
    pub fn new(a: A, b: B) -> Self {
        Self {a: a, b: b}
    }
}
impl<A, B, F> Element for Pipe<A, B>
where F: Freq,
      A: Element<Freq=F>,
      B: Element<Freq=F>,
      A::Src: Into<B::Sink> {
    type Sink = A::Sink;
    type Src = B::Src;
    type Freq = F;
    fn next(&mut self, sink: Self::Sink) -> Self::Src {
        self.b.next(self.a.next(sink).into())
    }
}
impl<A, B> Pipeline for Pipe<A, B>
where Self: Element<Sink=(), Src=()> {
    fn start(mut self) {
        loop {
            self.next(());
        }
    }
}
impl<A, B> SinkPipeline for Pipe<A, B>
where A: Element<Sink=()> + Send + Sync + 'static,
      B: PullElement<Sink=A::Src> {
    fn start(mut self) {
        self.b.start(self.a);
    }
}

// core macros

#[macro_export]
macro_rules! pipe {
    ( $e1:expr, $e2:expr ) => {
        Pipe::new($e1, $e2);
    };
    ( $e1:expr, $e2:expr, $( $e:expr ),* ) => {{
        let p = Pipe::new($e1, $e2);
        $(
            let p = Pipe::new(p, $e);
        )*
        p
    }}
}

pub struct FreqConv<E, F> {
    source: E,
    freq: ::std::marker::PhantomData<F>,
    buffer: f64,
    buffer_prev: f64,
    buffer_ptr: isize,
    next_ptr: isize
}
impl<E, F> FreqConv<E, F> {
    pub fn new(source: E) -> Self {
        Self {
            source: source,
            freq: ::std::marker::PhantomData,
            buffer: 0f64,
            buffer_prev: 0f64,
            buffer_ptr: -1,
            next_ptr: -1
        }
    }
}
impl<E: Element, F: Freq> Element for FreqConv<E, F> {
    type Sink = E::Sink;
    type Src = E::Src;
    type Freq = F;
    fn next(&mut self, sink: Self::Sink) -> Self::Src {
        self.source.next(sink)
    }
}

pub trait Sample {
    const MIN_LEVEL: Self;
    const MAX_LEVEL: Self;
    const REF_LEVEL: Self;
}
impl Sample for i32 {
    const MIN_LEVEL: Self = std::i32::MIN;
    const MAX_LEVEL: Self = std::i32::MAX;
    const REF_LEVEL: Self = 0i32;
}
impl Sample for f64 {
    const MIN_LEVEL: Self = -1f64;
    const MAX_LEVEL: Self = 1f64;
    const REF_LEVEL: Self = 0f64;
}

pub trait Freq {
    fn get_hz(&self) -> u32;
    fn get_period(&self) -> f64 {
        1f64 / self.get_hz() as f64
    }
}
pub struct AnyFreq {
    f: u32
}
impl AnyFreq {
    fn new(f: u32) -> Self {
        Self {f: f}
    }
}
impl Freq for AnyFreq {
    fn get_hz(&self) -> u32 {
        self.f
    }
}
pub trait ConstFreq : Default {
    const F: u32;
}
impl<T: ConstFreq> Freq for T {
    fn get_hz(&self) -> u32 {
        Self::F
    }
}
#[derive(Default)]
pub struct F44100;
impl ConstFreq for F44100 {
    const F: u32 = 44100;
}
#[derive(Default)]
pub struct F48000;
impl ConstFreq for F48000 {
    const F: u32 = 48000;
}
