use crate::Context;

use std::collections::HashSet;

pub struct DefaultContext {
    freq: Option<u32>,
    supported_freq: Option<HashSet<u32>>,
    preferred_freq: HashSet<u32>,
}
impl DefaultContext {
    pub fn new() -> Self {
        DefaultContext {
            freq: None,
            supported_freq: None,
            preferred_freq: HashSet::new(),
        }
    }
    pub fn freq(self, freq: u32) -> Self {
        DefaultContext {
            freq: Some(freq),
            ..self
        }
    }
}
impl Context for DefaultContext {
    fn build(mut self) -> Result<Self, ()> {
        self.decide_freq()?;
        Ok(self)
    }
}

pub trait FreqCtx {
    fn get_freq(&self) -> u32;
    fn set_supported_freq(&mut self, supported_freq: &[u32]);
    fn set_preferred_freq(&mut self, preferred_freq: &[u32]);
    fn decide_freq(&mut self) -> Result<u32, ()>;
}
impl FreqCtx for DefaultContext {
    fn get_freq(&self) -> u32 {
        self.freq.unwrap()
    }
    fn set_supported_freq(&mut self, supported_freq: &[u32]) {
        let supported_freq = supported_freq.iter().copied().collect();
        if let Some(ref mut self_supported_freq) = self.supported_freq {
            self_supported_freq.intersection(&supported_freq);
        } else {
            self.supported_freq = Some(supported_freq);
        }
    }
    fn set_preferred_freq(&mut self, preferred_freq: &[u32]) {
        self.preferred_freq.extend(preferred_freq.iter());
    }
    fn decide_freq(&mut self) -> Result<u32, ()> {
        if let Some(freq) = self.freq {
            return Ok(freq);
        }
        if let Some(supported_freq) = &self.supported_freq {
            self.preferred_freq.intersection(supported_freq);
            if let Some(max) = self.preferred_freq.iter().copied().max() {
                self.freq = Some(max);
            } else if let Some(max) = supported_freq.iter().copied().max() {
                self.freq = Some(max);
            }
        } else {
            if let Some(max) = self.preferred_freq.iter().copied().max() {
                self.freq = Some(max);
            }
        }
        if let Some(freq) = self.freq {
            return Ok(freq);
        } else {
            return Err(());
        }
    }
}
