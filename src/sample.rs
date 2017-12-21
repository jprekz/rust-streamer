#[derive(Copy, Clone, Debug)]
pub struct Stereo<T: SampleType> {
    pub l: T,
    pub r: T
}

#[derive(Copy, Clone, Debug)]
pub struct Mono<T: SampleType>(T);

pub trait Sample : Copy {
    type Member: SampleType;
    fn to_stereo(self) -> Stereo<Self::Member>;
    fn to_mono(self) -> Mono<Self::Member>;
    fn from_raw(raw: &[Self::Member]) -> Option<Self>;
}

impl<T: SampleType> Sample for Stereo<T> {
    type Member = T;
    fn to_stereo(self) -> Stereo<Self::Member> {
        self
    }
    fn to_mono(self) -> Mono<Self::Member> {
        Mono((self.l + self.r) / Self::Member::from_i32(2))
    }
    fn from_raw(raw: &[Self::Member]) -> Option<Self> {
        if raw.len() != 2 { return None; }
        Some(Stereo {
            l: raw[0],
            r: raw[1]
        })
    }
}

impl<T: SampleType> Sample for Mono<T> {
    type Member = T;
    fn to_stereo(self) -> Stereo<Self::Member> {
        Stereo {
            l: self.0,
            r: self.0
        }
    }
    fn to_mono(self) -> Mono<Self::Member> {
        self
    }
    fn from_raw(raw: &[Self::Member]) -> Option<Self> {
        if raw.len() != 1 { return None; }
        Some(Mono(raw[0]))
    }
}

pub trait IntoSample<T> {
    fn into_sample(self) -> T;
}
impl<S1, S2> IntoSample<Stereo<S2>> for Stereo<S1>
where S1: SampleType + IntoSampleType<S2>,
      S2: SampleType {
    fn into_sample(self) -> Stereo<S2> {
        Stereo { l: self.l.into_sampletype(), r: self.r.into_sampletype() }
    }
}
impl<S1, S2> IntoSample<Mono<S2>> for Stereo<S1>
where S1: SampleType + IntoSampleType<S2>,
      S2: SampleType {
    fn into_sample(self) -> Mono<S2> {
        self.to_mono().into_sample()
    }
}
impl<S1, S2> IntoSample<Mono<S2>> for Mono<S1>
where S1: SampleType + IntoSampleType<S2>,
      S2: SampleType {
    fn into_sample(self) -> Mono<S2> {
        Mono(self.0.into_sampletype())
    }
}
impl<S1, S2> IntoSample<Stereo<S2>> for Mono<S1>
where S1: SampleType + IntoSampleType<S2>,
      S2: SampleType {
    fn into_sample(self) -> Stereo<S2> {
        self.to_stereo().into_sample()
    }
}

use std::ops::*;
pub trait SampleType : Copy + Add<Output=Self> + Sub<Output=Self> + Mul<Output=Self> + Div<Output=Self> {
    const MIN_LEVEL: Self;
    const MAX_LEVEL: Self;
    const REF_LEVEL: Self;
    fn from_i32(i: i32) -> Self;
}
impl SampleType for i16 {
    const MIN_LEVEL: Self = ::std::i16::MIN;
    const MAX_LEVEL: Self = ::std::i16::MAX;
    const REF_LEVEL: Self = 0;
    fn from_i32(i: i32) -> Self {
        i as Self
    }
}
impl SampleType for i32 {
    const MIN_LEVEL: Self = ::std::i32::MIN;
    const MAX_LEVEL: Self = ::std::i32::MAX;
    const REF_LEVEL: Self = 0;
    fn from_i32(i: i32) -> Self {
        i as Self
    }
}
impl SampleType for f32 {
    const MIN_LEVEL: Self = -1f32;
    const MAX_LEVEL: Self = 1f32;
    const REF_LEVEL: Self = 0f32;
    fn from_i32(i: i32) -> Self {
        i as Self
    }
}
impl SampleType for f64 {
    const MIN_LEVEL: Self = -1f64;
    const MAX_LEVEL: Self = 1f64;
    const REF_LEVEL: Self = 0f64;
    fn from_i32(i: i32) -> Self {
        i as Self
    }
}

pub trait IntoSampleType<T: SampleType> : SampleType {
    fn into_sampletype(self) -> T;
}
impl<T: SampleType> IntoSampleType<T> for T {
    fn into_sampletype(self) -> T {
        self
    }
}
impl IntoSampleType<i32> for i16 {
    fn into_sampletype(self) -> i32 {
        self as i32 * (i32::MAX_LEVEL / i16::MAX_LEVEL as i32)
    }
}
impl IntoSampleType<f32> for i16 {
    fn into_sampletype(self) -> f32 {
        self as f32 / i16::MAX_LEVEL as f32
    }
}
impl IntoSampleType<f64> for i16 {
    fn into_sampletype(self) -> f64 {
        self as f64 / i16::MAX_LEVEL as f64
    }
}
impl IntoSampleType<i16> for i32 {
    fn into_sampletype(self) -> i16 {
        (self / (i32::MAX_LEVEL / i16::MAX_LEVEL as i32)) as i16
    }
}
impl IntoSampleType<f32> for i32 {
    fn into_sampletype(self) -> f32 {
        self as f32 / i32::MAX_LEVEL as f32
    }
}
impl IntoSampleType<f64> for i32 {
    fn into_sampletype(self) -> f64 {
        self as f64 / i32::MAX_LEVEL as f64
    }
}
impl IntoSampleType<i16> for f32 {
    fn into_sampletype(self) -> i16 {
        (self * i16::MAX_LEVEL as f32) as i16
    }
}
impl IntoSampleType<i32> for f32 {
    fn into_sampletype(self) -> i32 {
        (self * i32::MAX_LEVEL as f32) as i32
    }
}
impl IntoSampleType<f64> for f32 {
    fn into_sampletype(self) -> f64 {
        self as f64
    }
}
impl IntoSampleType<i16> for f64 {
    fn into_sampletype(self) -> i16 {
        (self * i16::MAX_LEVEL as f64) as i16
    }
}
impl IntoSampleType<i32> for f64 {
    fn into_sampletype(self) -> i32 {
        (self * i32::MAX_LEVEL as f64) as i32
    }
}
impl IntoSampleType<f32> for f64 {
    fn into_sampletype(self) -> f32 {
        self as f32
    }
}
