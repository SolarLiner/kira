use std::sync::Arc;

use atomic_arena::{Arena, Controller};
use ringbuf::Producer;

use crate::{
	manager::{backend::context::Context, command::ParameterCommand},
	parameter::{Parameter, ParameterId},
};

pub(crate) struct Parameters {
	parameters: Arena<Parameter>,
	unused_parameter_producer: Producer<Parameter>,
}

impl Parameters {
	pub fn new(capacity: usize, unused_parameter_producer: Producer<Parameter>) -> Self {
		Self {
			parameters: Arena::new(capacity),
			unused_parameter_producer,
		}
	}

	pub fn controller(&self) -> Controller {
		self.parameters.controller()
	}

	pub fn get(&self, id: ParameterId) -> Option<&Parameter> {
		self.parameters.get(id.0)
	}

	fn remove_unused_parameters(&mut self) {
		if self.unused_parameter_producer.is_full() {
			return;
		}
		for (_, parameter) in self
			.parameters
			.drain_filter(|parameter| parameter.shared().is_marked_for_removal())
		{
			if self.unused_parameter_producer.push(parameter).is_err() {
				panic!("Unused parameter producer is full")
			}
			if self.unused_parameter_producer.is_full() {
				return;
			}
		}
	}

	pub fn on_start_processing(&mut self) {
		self.remove_unused_parameters();
		for (_, parameter) in &self.parameters {
			parameter.on_start_processing();
		}
	}

	pub fn run_command(&mut self, command: ParameterCommand, context: &Arc<Context>) {
		match command {
			ParameterCommand::Add(id, parameter) => self
				.parameters
				.insert_with_index(id.0, parameter)
				.expect("Parameter arena is full"),
			ParameterCommand::Set(id, value) => {
				if let Some(parameter) = self.parameters.get_mut(id.0) {
					parameter.set(value);
				}
			}
			ParameterCommand::Tween {
				id,
				target,
				tween,
				command_sent_time,
			} => {
				if let Some(parameter) = self.parameters.get_mut(id.0) {
					parameter.tween(context, target, tween, command_sent_time)
				}
			}
		}
	}

	pub fn update(&mut self, dt: f64) {
		for (_, parameter) in &mut self.parameters {
			parameter.update(dt);
		}
	}
}