use std::{env, error::Error, io};

include!(concat!(env!("OUT_DIR"), "/", "markov.rs"));

fn main() -> Result<(), Box<dyn Error>> {
	let content = markov::generate();
	let suffix = option_env!("SHITPOST_SUFFIX").unwrap_or_default();

	let mut private_key_buf = String::new();
	let private_key = env::args().nth(1);

	let private_key = match private_key.as_deref() {
		Some("-") => {
			eprintln!("note: reading private key from stdin.\n");
			io::stdin().read_line(&mut private_key_buf)?;
			private_key_buf.trim()
		},
		Some(private_key) => private_key,
		None => {
			#[cfg(not(no_send))]
			eprintln!("note: no private key received, printing output instead.\n");
			println!("{content}{suffix}");
			return Ok(());
		},
	};

	#[cfg(not(no_send))]
	{
		let content = content.replace('"', r#"\""#);
		let content = if content.ends_with("</p>") {
			&content[..content.len() - 4]
		} else {
			&content
		};
		let close = if content.starts_with("<p>") {
			"</p>"
		} else {
			""
		};
		reqwest::blocking::Client::new()
			.post(env!("SHITPOST_OUT"))
			.header("Authorization", format!("Bearer {private_key}"))
			.header("Content-Type", "application/json")
			.body(format!(
				r#"
                {{
					{}
                    "content_type": "text/markdown",
                    "status": "{content}{suffix}{close}",
					"visibility": "{}"
                }}
                "#,
				option_env!("SHITPOST_CW")
					.map(|cw| format!(r#""spoiler_text": "{cw}","#))
					.unwrap_or(String::new()),
				option_env!("SHITPOST_VISIBILITY")
					.unwrap_or("public"),
			))
			.send()?
			.error_for_status()?;
	}

	#[cfg(no_send)]
	{
		let _ = private_key;
		eprintln!("Not compiled with support for sending posts. Run without args to just print the output");
	}

	Ok(())
}