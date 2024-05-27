use std::{env, error::Error, path::PathBuf};

use shitpost_markov::Markov;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
	println!("cargo::rerun-if-env-changed=SHITPOST_IN");
	println!("cargo::rerun-if-env-changed=SHITPOST_KEY");
	println!("cargo::rerun-if-env-changed=SHITPOST_OUT");
	println!("cargo::rerun-if-env-changed=SHITPOST_SUFFIX");
	println!("cargo::rerun-if-env-changed=SHITPOST_VISIBILITY");
	println!("cargo::rerun-if-env-changed=SHITPOST_CW");

	let outdir = PathBuf::from(env::var("OUT_DIR")?);

	let url = env::var("SHITPOST_IN").expect("No URL provided to train posts on, please set the SHITPOST_IN environment variable");
	let private_key = env::var("SHITPOST_KEY").ok();

	if env::var("SHITPOST_OUT").is_err() {
		println!("cargo::rustc-cfg=no_send");
		println!("cargo::warning=Environment variable 'SHITPOST_OUT' is not set. Compiling without support for sending.");
	}

	let mut builder =
		Markov::builder(&url).outdir(outdir.join("markov.rs"));

	if let Some(private_key) = private_key {
		builder = builder.private_key(private_key);
	}

	builder.build().await?.write_table(&outdir)?;

	Ok(())
}
