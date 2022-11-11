use dasp::Sample;

use crate::output::OutputSample;
use crate::application::Float;


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
    I::Item: OutputSample
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

    fn interpolate(&self, t: Float) -> Option<I::Item> {
        if let Some(a) = self.prev {
            let a = a.to_sample::<Float>();
            if let Some(b) = self.next {
                let b = b.to_sample::<Float>();
                return Some(((1.0 - t) * a + t * b).to_sample::<I::Item>());
            } else {
                return Some(((1.0 - t) * a).to_sample::<I::Item>());
            }
        }
        None
    }
}

impl<I> Iterator for LinearInterpolator<I>
where
    I: Iterator,
    I::Item: OutputSample
{
    type Item = I::Item;

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
