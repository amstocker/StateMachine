use dasp::{Sample, sample::{FromSample, ToSample}};

use crate::sound::Float;


pub struct LinearInterpolator<I> where I: Iterator {
    iterator: I,
    prev: Option<I::Item>,
    next: Option<I::Item>,
    ratio: Float,
    step: Float
}

impl<I> LinearInterpolator<I>
where
    I: Iterator,
    I::Item: Sample + ToSample<Float> + FromSample<Float>
{
    pub fn new(mut iterator: I, ratio: Float) -> Self {
        let prev = iterator.next();
        let next = iterator.next();
        Self {
            iterator,
            prev,
            next,
            ratio,
            step: 0.0
        }
    }

    fn interpolate(&self, t: Float) -> Option<Float> {
        if let Some(a) = self.prev {
            let a = a.to_sample::<Float>();
            if let Some(b) = self.next {
                let b = b.to_sample::<Float>();
                return Some((1.0 - t) * a + t * b);
            } else {
                return Some((1.0 - t) * a);
            }
        }
        None
    }
}

impl<I> Iterator for LinearInterpolator<I>
where
    I: Iterator,
    I::Item: Sample + ToSample<Float> + FromSample<Float>
{
    type Item = Float;

    fn next(&mut self) -> Option<Self::Item> {
        if self.step >= self.ratio {
            self.step -= self.ratio;
            self.prev = self.next;
            self.next = self.iterator.next();
        }
        let result = self.interpolate(self.step / self.ratio);
        self.step += 1.0;
        result
    }
}
