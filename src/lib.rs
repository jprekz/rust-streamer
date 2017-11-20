pub mod wav;
pub mod element;

// core traits

pub trait Element {
    type Sink;
    type Src;
    fn next(&mut self, sink: Self::Sink) -> Self::Src;
}
pub trait PullElement {
    type Sink;
    fn start<E>(&mut self, sink: E)
        where E: Element<Sink=(), Src=Self::Sink> + Send + Sync + 'static;
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
impl<A, B> Element for Pipe<A, B>
where A: Element,
      B: Element,
      A::Src: Into<B::Sink> {
    type Sink = A::Sink;
    type Src = B::Src;
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
