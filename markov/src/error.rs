use std::sync::PoisonError;

use actson::{
	feeder::FillError,
	parser::{InvalidStringValueError, ParserError},
};
use thiserror::Error;
use tokio::{io, task::JoinError};

use crate::tokens::Tokens;

#[derive(Debug, Error)]
pub enum Error {
	#[error("Request failed: {0}")]
	RequestError(#[from] reqwest::Error),
	#[error(":(")]
	JoinError(#[from] JoinError),
	#[error(":(")]
	IOError(#[from] io::Error),
	#[error(":(")]
	PoisonError(#[from] PoisonError<Tokens>),
	#[error("A reference existed for too long.")]
	Arc,
	#[error("")]
	InternerPoison,
	#[error("")]
	InternerError,
	#[error("Failed to give input to JSON parser: {0}")]
	JsonFillError(#[from] FillError),
	#[error("JSON value is not a string: {0}")]
	JsonStringError(#[from] InvalidStringValueError),
	#[error("Failed to parse JSON response: {0}")]
	JsonParseError(#[from] ParserError),
}
