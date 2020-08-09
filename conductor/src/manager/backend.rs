use super::{AudioManagerSettings, Event};
use crate::{
	command::Command,
	instance::{Instance, InstanceId},
	project::Project,
	sequence::{Sequence, SequenceId},
	stereo_sample::StereoSample,
};
use indexmap::IndexMap;
use ringbuf::{Consumer, Producer};

pub struct Backend {
	dt: f32,
	project: Project,
	instances: IndexMap<InstanceId, Instance>,
	sequences: IndexMap<SequenceId, Sequence>,
	command_consumer: Consumer<Command>,
	event_producer: Producer<Event>,

	metronome_interval_event_collector: Vec<f32>,
	sequence_command_queue: Vec<Command>,
	sequences_to_remove: Vec<SequenceId>,
	instances_to_remove: Vec<InstanceId>,
}

impl Backend {
	pub fn new(
		sample_rate: u32,
		project: Project,
		command_consumer: Consumer<Command>,
		event_producer: Producer<Event>,
		settings: AudioManagerSettings,
	) -> Self {
		Self {
			dt: 1.0 / sample_rate as f32,
			project,
			instances: IndexMap::with_capacity(settings.num_instances),
			sequences: IndexMap::with_capacity(settings.num_sequences),
			command_consumer,
			event_producer,
			metronome_interval_event_collector: Vec::with_capacity(settings.num_events),
			sequence_command_queue: Vec::with_capacity(settings.num_commands),
			sequences_to_remove: Vec::with_capacity(settings.num_sequences),
			instances_to_remove: Vec::with_capacity(settings.num_instances),
		}
	}

	fn run_command(&mut self, command: Command) {
		match command {
			Command::PlaySound(sound_id, instance_id, settings) => {
				let sound = self.project.sounds.get(&sound_id).unwrap();
				self.instances.insert(
					instance_id,
					Instance::new(sound_id, settings, sound.duration()),
				);
			}
			Command::PauseInstance(instance_id, fade_duration) => {
				if let Some(instance) = self.instances.get_mut(&instance_id) {
					instance.pause(fade_duration);
				}
			}
			Command::ResumeInstance(instance_id, fade_duration) => {
				if let Some(instance) = self.instances.get_mut(&instance_id) {
					instance.resume(fade_duration);
				}
			}
			Command::StopInstance(instance_id, fade_duration) => {
				if let Some(instance) = self.instances.get_mut(&instance_id) {
					instance.stop(fade_duration);
				}
			}
			Command::StartMetronome(id) => {
				self.project.metronomes.get_mut(&id).unwrap().start();
			}
			Command::PauseMetronome(id) => {
				self.project.metronomes.get_mut(&id).unwrap().pause();
			}
			Command::StopMetronome(id) => {
				self.project.metronomes.get_mut(&id).unwrap().stop();
			}
			Command::StartSequence(id, mut sequence) => {
				let metronome = self.project.metronomes.get(&sequence.metronome_id).unwrap();
				sequence.start(metronome, &mut self.sequence_command_queue);
				self.sequences.insert(id, sequence);
			}
		}
	}

	pub fn process_commands(&mut self) {
		while let Some(command) = self.command_consumer.pop() {
			self.run_command(command);
		}
	}

	pub fn update_metronomes(&mut self) {
		for (id, metronome) in &mut self.project.metronomes {
			metronome.update(self.dt, &mut self.metronome_interval_event_collector);
			for interval in self.metronome_interval_event_collector.drain(..) {
				match self
					.event_producer
					.push(Event::MetronomeIntervalPassed(*id, interval))
				{
					Ok(_) => {}
					Err(_) => {}
				}
			}
		}
	}

	pub fn update_sequences(&mut self) {
		for (id, sequence) in &mut self.sequences {
			let metronome = self.project.metronomes.get(&sequence.metronome_id).unwrap();
			sequence.update(self.dt, &metronome, &mut self.sequence_command_queue);
			if sequence.finished() {
				self.sequences_to_remove.push(*id);
			}
		}
		for id in self.sequences_to_remove.drain(..) {
			self.sequences.remove(&id);
		}
		for i in 0..self.sequence_command_queue.len() {
			let command = self.sequence_command_queue.get(i).unwrap().clone();
			self.run_command(command);
		}
		self.sequence_command_queue.clear();
	}

	pub fn process(&mut self) -> StereoSample {
		self.process_commands();
		self.update_metronomes();
		self.update_sequences();
		let mut out = StereoSample::from_mono(0.0);
		for (instance_id, instance) in &mut self.instances {
			if instance.playing() {
				let sound = self.project.sounds.get(&instance.sound_id).unwrap();
				out +=
					sound.get_sample_at_position(instance.position()) * instance.effective_volume();
			}
			if instance.finished() {
				self.instances_to_remove.push(*instance_id);
			}
			instance.update(self.dt);
		}
		for instance_id in self.instances_to_remove.drain(..) {
			self.instances.remove(&instance_id);
		}
		out
	}
}
