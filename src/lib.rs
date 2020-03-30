use std::ops::Add;

pub mod context;
pub mod dsp;
pub mod element;
#[cfg(feature = "graphic")]
pub mod graphic;
pub mod sample;
pub mod wav;

// core traits

pub trait Element<Sink, Ctx> {
    type Src;
    fn init(&mut self, _ctx: &mut Ctx) {}
    fn start(&mut self, _ctx: &Ctx) {}
    fn next(&mut self, sink: Sink, ctx: &Ctx) -> Self::Src;
}
pub trait PullElement<Sink, Ctx> {
    fn init(&mut self, _ctx: &mut Ctx) {}
    fn start(&mut self, sink: impl Element<(), Ctx, Src = Sink> + Send, ctx: &Ctx);
    fn stop(&mut self);
}
pub trait PushElement<Ctx> {
    type Src;
    fn init(&mut self, _ctx: &mut Ctx) {}
    fn start(&mut self, src: impl Element<Self::Src, Ctx, Src = ()> + Send, ctx: &Ctx);
    fn stop(&mut self);
}
pub trait Pipeline<Ctx: Context> {
    fn start(self, ctx: Ctx);
}
pub trait SinkPipeline<Ctx: Context> {
    fn start(self, ctx: Ctx);
}
pub trait SrcPipeline<Ctx: Context> {
    fn start(self, ctx: Ctx);
}
pub trait Context: Sized {
    fn build(self) -> Result<Self, ()>;
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
    fn init(&mut self, ctx: &mut Ctx) {
        self.a.init(ctx);
        self.b.init(ctx);
    }
    fn start(&mut self, ctx: &Ctx) {
        self.a.start(ctx);
        self.b.start(ctx);
    }
    fn next(&mut self, sink: Sink, ctx: &Ctx) -> Self::Src {
        self.b.next(self.a.next(sink, ctx), ctx)
    }
}
impl<A, B, Ctx: Context> Pipeline<Ctx> for Pipe<A, B>
where
    Self: Element<(), Ctx, Src = ()>,
{
    fn start(mut self, mut ctx: Ctx) {
        self.init(&mut ctx);
        let ctx = ctx.build().unwrap();
        Element::start(&mut self, &ctx);
        loop {
            self.next((), &ctx);
        }
    }
}
impl<A, B, Ctx: Context> SinkPipeline<Ctx> for Pipe<A, B>
where
    A: Element<(), Ctx> + Send,
    B: PullElement<A::Src, Ctx>,
{
    fn start(mut self, mut ctx: Ctx) {
        self.a.init(&mut ctx);
        self.b.init(&mut ctx);
        let ctx = ctx.build().unwrap();
        self.a.start(&ctx);
        self.b.start(self.a, &ctx);
    }
}
impl<A, B, Ctx: Context> SrcPipeline<Ctx> for Pipe<A, B>
where
    A: PushElement<Ctx>,
    B: Element<A::Src, Ctx, Src = ()> + Send,
{
    fn start(mut self, mut ctx: Ctx) {
        self.a.init(&mut ctx);
        self.b.init(&mut ctx);
        let ctx = ctx.build().unwrap();
        self.b.start(&ctx);
        self.a.start(self.b, &ctx);
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

// fork

pub struct Fork<A, B> {
    a: A,
    b: B,
}
impl<A, B> Fork<A, B> {
    pub fn new(a: A, b: B) -> Self {
        Self { a: a, b: b }
    }
}
impl<A, B, Sink, Ctx> Element<Sink, Ctx> for Fork<A, B>
where
    A: Element<Sink, Ctx>,
    B: Element<Sink, Ctx>,
    A::Src: Add<B::Src>,
    Sink: Copy,
{
    type Src = <A::Src as Add<B::Src>>::Output;
    fn init(&mut self, ctx: &mut Ctx) {
        self.a.init(ctx);
        self.b.init(ctx);
    }
    fn start(&mut self, ctx: &Ctx) {
        self.a.start(ctx);
        self.b.start(ctx);
    }
    fn next(&mut self, sink: Sink, ctx: &Ctx) -> Self::Src {
        self.a.next(sink, ctx) + self.b.next(sink, ctx)
    }
}

#[macro_export]
macro_rules! fork {
    ( $e1:expr, $e2:expr ) => {
        Fork::new($e1, $e2);
    };
    ( $e1:expr, $e2:expr, $( $e:expr ),* ) => {{
        let f = Fork::new($e1, $e2);
        $(
            let f = Fork::new(f, $e);
        )*
        f
    }}
}
