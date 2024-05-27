use std::path::PathBuf;

use tokio::fs;

use crate::{error::Error, Markov};

pub struct Builder<'a> {
	url: &'a str,
	outdir: Option<PathBuf>,
	private_key: Option<String>,
	exclude_replies: bool,
}

impl<'a> Builder<'a> {
	#[must_use]
	pub fn new(url: &'a str) -> Builder {
		Self {
			url,
			outdir: None,
			private_key: None,
			exclude_replies: false,
		}
	}

	#[must_use]
	pub fn outdir(self, outdir: PathBuf) -> Self {
		Self {
			outdir: Some(outdir),
			..self
		}
	}

	#[must_use]
	pub fn exclude_replies(self) -> Self {
		Self {
			exclude_replies: true,
			..self
		}
	}

	#[must_use]
	pub fn private_key(self, key: String) -> Self {
		Self {
			private_key: Some(key),
			..self
		}
	}

	pub async fn build(self) -> Result<Markov, Error> {
		if let Some(path) = self.outdir {
			fs::write(path, include_bytes!("markov.rs")).await?;
		}
		Markov::new(self.url, self.private_key, self.exclude_replies)
			.await
	}
}
