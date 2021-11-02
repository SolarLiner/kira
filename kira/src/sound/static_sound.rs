use std::{sync::Arc, time::Duration};

use crate::{
	dsp::Frame,
	manager::resources::{Clocks, Parameters},
	track::TrackId,
	util,
};

use super::{ProcessResult, Sound};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlaybackState {
	Playing,
	Pausing,
	Stopped,
}

#[derive(Debug, Clone)]
pub struct StaticSound<S: Into<Frame> + Send + Sync + Copy> {
	sample_rate: u32,
	samples: Arc<Vec<S>>,
	state: PlaybackState,
	position: f64,
}

impl<S: Into<Frame> + Send + Sync + Copy> StaticSound<S> {
	pub fn new(sample_rate: u32, samples: Vec<S>) -> Self {
		Self {
			sample_rate,
			samples: Arc::new(samples),
			state: PlaybackState::Playing,
			position: 0.0,
		}
	}

	pub fn duration(&self) -> Duration {
		Duration::from_secs_f64(self.samples.len() as f64 / self.sample_rate as f64)
	}

	fn frame_at_index(&self, index: usize) -> Frame {
		self.samples
			.get(index)
			.copied()
			.map(|sample| sample.into())
			.unwrap_or(Frame::ZERO)
	}

	pub fn frame_at_position(&self, position: f64) -> Frame {
		let sample_position = self.sample_rate as f64 * position;
		let fraction = (sample_position % 1.0) as f32;
		let current_sample_index = sample_position as usize;
		let previous = if current_sample_index == 0 {
			Frame::ZERO
		} else {
			self.frame_at_index(current_sample_index - 1)
		};
		let current = self.frame_at_index(current_sample_index);
		let next_1 = self.frame_at_index(current_sample_index + 1);
		let next_2 = self.frame_at_index(current_sample_index + 2);
		util::interpolate_frame(previous, current, next_1, next_2, fraction)
	}
}

impl<S: Into<Frame> + Send + Sync + Copy> Sound for StaticSound<S> {
	fn sample_rate(&mut self) -> u32 {
		self.sample_rate
	}

	fn track(&mut self) -> TrackId {
		TrackId::Main
	}

	fn process(&mut self, dt: f64, _parameters: &Parameters, _clocks: &Clocks) -> ProcessResult {
		let out = self.frame_at_position(self.position);
		if self.position > self.duration().as_secs_f64() {
			self.state = PlaybackState::Stopped;
		}
		self.position += dt;
		ProcessResult::Loaded(out)
	}

	fn finished(&mut self) -> bool {
		self.state == PlaybackState::Stopped
	}
}
