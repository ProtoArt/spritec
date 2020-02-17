use std::cmp::min;

use interpolation::lerp;

use crate::math::Milliseconds;
use crate::query3d::AnimationPosition;

use super::interpolate::{Interpolation, Interpolate};

pub enum KeyframeRange<'a, T> {
    /// The keyframe before the specified time
    Before(&'a Frame<T>),
    /// The keyframes that immediately surround the specified time
    Between(&'a Frame<T>, &'a Frame<T>),
    /// The keyframe after the specified time
    After(&'a Frame<T>),
}

#[derive(Debug)]
pub struct Keyframes<T> {
    pub frames: Vec<Frame<T>>,
    pub interpolation: Interpolation,
}

impl<T> Keyframes<T> {
    /// Retrieves the keyframes immediately surrounding the given time
    /// A time smaller than that of all keyframes will get back the first keyframe twice
    /// A time larger than all keyframes gets the last keyframe twice
    pub fn surrounding(&self, time: Milliseconds) -> KeyframeRange<T> {
        // This unwrap is safe for partial_cmp as long as NaN is not one of the comparison values
        let index = match self.frames.binary_search_by(|frame| frame.time.partial_cmp(&time).unwrap()) {
            Ok(i) | Err(i) => i,
        };

        if index == 0 {
            KeyframeRange::After(&self.frames[index])
        } else if index == self.frames.len() {
            KeyframeRange::Before(&self.frames[index - 1])
        } else {
            let left_index = index - 1;
            let right_index = min(index, self.frames.len() - 1);
            KeyframeRange::Between(&self.frames[left_index], &self.frames[right_index])
        }
    }

    pub fn end_time(&self) -> Milliseconds {
        let last_index = self.frames.len() - 1;
        self.frames[last_index].time
    }

    pub fn value_at(&self, pos: &AnimationPosition) -> T
        where T: Interpolate + Copy,
    {
        let time = match pos {
            &AnimationPosition::Time(t) => t,
            &AnimationPosition::RelativeTime{start_time, weight} => {
                Milliseconds::from_msec(lerp(&start_time.to_msec(), &self.end_time().to_msec(), &weight))
            },
        };

        let new_value = match self.surrounding(time) {
            KeyframeRange::Before(kf) => kf.value,
            KeyframeRange::After(kf) => kf.value,
            KeyframeRange::Between(kf1, kf2) => {
                let start = kf1.time;
                let end = kf2.time;
                // The time factor that gives weight to the start or end frame during interpolation
                let weight = (time.to_msec() - start.to_msec()) / (end.to_msec() - start.to_msec());
                T::interpolate(self.interpolation, weight, &kf1.value, &kf2.value)
            },
        };

        new_value
    }
}

#[derive(Debug)]
pub struct Frame<T> {
    pub time: Milliseconds,
    pub value: T,
}
