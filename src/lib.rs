pub mod element;
pub mod wav;
pub mod sample;
pub mod dsp;

// core traits

pub trait Element<Sink, Ctx> {
    type Src;
    fn next(&mut self, sink: Sink, ctx: &Ctx) -> Self::Src;
}
pub trait PullElement<Sink, Ctx> {
    fn start<E>(&mut self, sink: E, ctx: &Ctx)
    where
        E: Element<(), Ctx, Src = Sink> + Send;
    fn stop(&mut self);
}
pub trait PushElement<Ctx> {
    type Src;
    fn start<E>(&mut self, src: E, ctx: &Ctx)
    where
        E: Element<Self::Src, Ctx, Src = ()> + Send;
    fn stop(&mut self);
}
pub trait Pipeline<Ctx> {
    fn start(self, ctx: &Ctx);
}
pub trait SinkPipeline<Ctx> {
    fn start(self, ctx: &Ctx);
}
pub trait SrcPipeline<Ctx> {
    fn start(self, ctx: &Ctx);
}

// pipe

pub struct Pipe<A, B> {
    a: A,
    b: B,
}
impl<A, B> Pipe<A, B> {
    pub fn new(a: A, b: B) -> Self {
        Self { a: a, b: b }
    }
}
impl<A, B, Sink, Ctx> Element<Sink, Ctx> for Pipe<A, B>
where
    A: Element<Sink, Ctx>,
    B: Element<A::Src, Ctx>,
{
    type Src = B::Src;
    fn next(&mut self, sink: Sink, ctx: &Ctx) -> Self::Src {
        self.b.next(self.a.next(sink, ctx), ctx)
    }
}
impl<A, B, Ctx> Pipeline<Ctx> for Pipe<A, B>
where
    Self: Element<(), Ctx, Src = ()>,
{
    fn start(mut self, ctx: &Ctx) {
        loop {
            self.next((), ctx);
        }
    }
}
impl<A, B, Ctx> SinkPipeline<Ctx> for Pipe<A, B>
where
    A: Element<(), Ctx> + Send,
    B: PullElement<A::Src, Ctx>,
{
    fn start(mut self, ctx: &Ctx) {
        self.b.start(self.a, ctx);
    }
}
impl<A, B, Ctx> SrcPipeline<Ctx> for Pipe<A, B>
where
    A: PushElement<Ctx>,
    B: Element<A::Src, Ctx, Src = ()> + Send,
{
    fn start(mut self, ctx: &Ctx) {
        self.a.start(self.b, ctx);
    }
}

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

// context

pub struct Context {
    freq: u32,
}
impl Context {
    pub fn new(freq: u32) -> Self {
        Self { freq: freq }
    }
}
pub trait FreqCtx {
    fn get_freq(&self) -> u32;
}
impl FreqCtx for Context {
    fn get_freq(&self) -> u32 {
        self.freq
    }
}

// other

pub trait FixedQueue {
    type T;
    fn push(&mut self, item: Self::T);
}
impl<T: Copy> FixedQueue for [T] {
    type T = T;
    fn push(&mut self, item: T) {
        let len = self.len();
        for i in 0..len - 1 {
            self[i] = self[i + 1];
        }
        self[len - 1] = item;
    }
}

pub struct FreqConv<E> {
    source: E,
    buffer: f64,
    buffer_prev: f64,
    buffer_ptr: isize,
    next_ptr: isize,
}
impl<E> FreqConv<E> {
    pub fn new(source: E) -> Self {
        Self {
            source: source,
            buffer: 0f64,
            buffer_prev: 0f64,
            buffer_ptr: -1,
            next_ptr: -1,
        }
    }
}
/*
impl<E: Element> Element for FreqConv<E> {
    type Sink = E::Sink;
    type Src = E::Src;
    fn next(&mut self, sink: Self::Sink) -> Self::Src {
        self.source.next(sink)
    }
}
*/
