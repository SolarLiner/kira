use crate::{
	instance::{InstanceId, InstanceSettings},
	sequence::{Sequence, SequenceId},
	sound::{Sound, SoundId},
	tempo::Tempo,
	tween::Tween,
};

pub(crate) enum SoundCommand {
	LoadSound(SoundId, Sound),
	UnloadSound(SoundId),
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum InstanceCommand<Id> {
	PlaySound(SoundId, Id, InstanceSettings),
	SetInstanceVolume(Id, f32, Option<Tween>),
	SetInstancePitch(Id, f32, Option<Tween>),
	PauseInstance(Id, Option<Tween>),
	ResumeInstance(Id, Option<Tween>),
	StopInstance(Id, Option<Tween>),
	PauseInstancesOfSound(SoundId, Option<Tween>),
	ResumeInstancesOfSound(SoundId, Option<Tween>),
	StopInstancesOfSound(SoundId, Option<Tween>),
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum MetronomeCommand {
	SetMetronomeTempo(Tempo),
	StartMetronome,
	PauseMetronome,
	StopMetronome,
}

pub(crate) enum SequenceCommand<CustomEvent> {
	StartSequence(SequenceId, Sequence<CustomEvent>),
}

pub(crate) enum Command<CustomEvent> {
	Sound(SoundCommand),
	Instance(InstanceCommand<InstanceId>),
	Metronome(MetronomeCommand),
	Sequence(SequenceCommand<CustomEvent>),
	EmitCustomEvent(CustomEvent),
}
