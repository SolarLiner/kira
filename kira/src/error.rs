//! Things that can go wrong when using an [`AudioManager`](super::AudioManager).

use cpal::{BuildStreamError, DefaultStreamConfigError, PlayStreamError};
use thiserror::Error;

/// Things that can go wrong when creating an `AudioManager`.
#[derive(Debug, Error)]
pub enum SetupError {
	/// A default audio output device could not be determined.
	#[error("Cannot find the default audio output device")]
	NoDefaultOutputDevice,

	/// An error occurred when getting the default output configuration.
	#[error("{0}")]
	DefaultStreamConfigError(#[from] DefaultStreamConfigError),

	/// An error occured when building the audio stream.
	#[error("{0}")]
	BuildStreamError(#[from] BuildStreamError),

	/// An error occured when starting the audio stream.
	#[error("{0}")]
	PlayStreamError(#[from] PlayStreamError),
}

#[derive(Debug, Error)]
pub enum CommandError {
	#[error("Could not add a sound because the command queue is full.")]
	CommandQueueFull,
	#[error("Could not add a sound because a thread panicked while using the command queue.")]
	MutexPoisoned,
}

#[derive(Debug, Error)]
pub enum AddSoundError {
	#[error("Could not add a sound because the maximum number of sounds has been reached.")]
	SoundLimitReached,
	#[error("{0}")]
	CommandError(#[from] CommandError),
}

#[cfg(any(feature = "mp3", feature = "ogg", feature = "flac", feature = "wav"))]
#[derive(Debug, Error)]
pub enum LoadSoundError {
	#[error("{0}")]
	FromFileError(#[from] crate::sound::data::static_sound::error::FromFileError),
	#[error("{0}")]
	AddSoundError(#[from] AddSoundError),
}

#[derive(Debug, Error)]
pub enum PlaySoundError {
	#[error("Could not add an instance because the maximum number of instances has been reached.")]
	InstanceLimitReached,
	#[error("{0}")]
	CommandError(#[from] CommandError),
}

#[derive(Debug, Error)]
pub enum AddParameterError {
	#[error(
		"Could not add a parameter because the maximum number of parameters has been reached."
	)]
	ParameterLimitReached,
	#[error("{0}")]
	CommandError(#[from] CommandError),
}

#[derive(Debug, Error)]
pub enum AddSubTrackError {
	#[error(
		"Could not add a sub-track because the maximum number of sub-tracks has been reached."
	)]
	SubTrackLimitReached,
	#[error("{0}")]
	CommandError(#[from] CommandError),
}

#[derive(Debug, Error)]
pub enum InstanceError {
	#[error("Cannot modify an instance that has finished playing")]
	InstanceStopped,
	#[error("{0}")]
	CommandError(#[from] CommandError),
}
