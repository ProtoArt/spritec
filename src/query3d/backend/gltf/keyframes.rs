use std::cmp::min;

use crate::math::Milliseconds;

use super::interpolate::Interpolation;

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
}

#[derive(Debug)]
pub struct Frame<T> {
    pub time: Milliseconds,
    pub value: T,
}
