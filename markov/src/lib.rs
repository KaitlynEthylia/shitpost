#![allow(clippy::missing_panics_doc, clippy::missing_errors_doc)]

mod builder;
mod error;
mod markov;
mod parse;
mod tokens;

use std::{
	cmp::Ordering,
	collections::HashMap,
	fs,
	path::Path,
	sync::{Arc, Mutex},
	time::Duration,
};

use markov::TokenType;
use rayon::prelude::*;
use reqwest::{Client, Method};
use tokens::Tokens;
use tokio::sync::mpsc;

use crate::tokens::INTERN;
pub use crate::{builder::Builder, error::Error};

pub struct Markov(Tokens);

const TOKEN_CAPACITY: usize = 2048;
const TOKEN_SUB_CAPACITY: usize = 256;

#[derive(PartialEq, Eq, Debug)]
enum ValueCategory {
	Overfull,
	Underfull,
	Done,
}

impl Markov {
	pub async fn new(
		url: &str,
		private_key: Option<String>,
		exclude_replies: bool,
	) -> Result<Markov, Error> {
		let tokens = Arc::new(Mutex::new(HashMap::with_capacity(
			TOKEN_CAPACITY,
		)));

		let (send, mut recv) = mpsc::unbounded_channel::<
			Result<Option<String>, Error>,
		>();

		let params = [
			("limit", "40"),
			("exclude_reblogs", "true"),
			("exclude_replies", &exclude_replies.to_string()),
		];

		let client = Client::new();
		let mut rq = client
			.request(Method::GET, url)
			.query(&params)
			.timeout(Duration::from_secs(20));

		if let Some(key) = private_key {
			rq = rq.header(
				"Authorization",
				"Bearer ".to_string() + &key,
			);
		}

		send.send(Ok(Some(String::from("0"))))
			.expect("Sending channel closed unexpectedly.");

		let mut tasks = Vec::new();
		while let Some(min_id) = recv.recv().await {
			let min_id = min_id?;
			match min_id {
				Some(min_id) => {
					let send = send.clone();
					let tokens = tokens.clone();
					let rq = rq.try_clone().expect(":(");
					tasks.push(tokio::spawn(async move {
						parse::process_posts(
							rq, min_id, send, tokens,
						)
						.await;
					}));
				},
				None => break,
			}
		}
		for task in tasks {
			task.await?;
		}

		let tokens = Arc::into_inner(tokens)
			.ok_or(Error::Arc)?
			.into_inner()?;
		Ok(Markov(tokens))
	}

	#[must_use]
	pub fn builder(url: &str) -> Builder {
		Builder::new(url)
	}

	#[allow(clippy::cast_precision_loss)]
	pub fn write_table(self, outdir: &Path) -> Result<(), Error> {
		let intern =
			INTERN.lock().map_err(|_| Error::InternerPoison)?;
		let indexes = self
			.0
			.iter()
			.zip(0..)
			.map(|((t, _), e)| (*t, e))
			.collect::<HashMap<TokenType, u16>>();
		let aliases = self
            .0
            .into_par_iter()
            .map(|(token, counts)| {
                let ty = match token {
                    TokenType::Tag { whitespace, tag, attrs } => format!(
                        r#####"Tag {{ whitespace: {:?}, tag: r####"{}"####, attrs: r####"{}"#### }}"#####,
                        whitespace.and_then(|w| intern.resolve(w)),
                        intern.resolve(tag).ok_or(Error::InternerError)?,
                        intern.resolve(attrs).ok_or(Error::InternerError)?
                    ),
                    TokenType::Text(s) => {
                        let s = intern.resolve(s).ok_or(Error::InternerError)?;
                        format!(r#####"Text(r####"{s}"####)"#####)
                    }
                    _ => format!("{token:?}"),
                };

                let mut table =
                    format!("\tcrate::Token {{ token: crate::TokenType::{ty}, alias: &[\n\t\t");
                // generate alias table

                let buckets = counts.len();
                let sum = counts.par_iter().map(|(_, c)| c).sum::<usize>() as f64;
                let mut counts = counts
                    .into_par_iter()
                    .map(|(token, count)| {
                        let k = count as f64 / sum * buckets as f64;

                        let cat = match k.total_cmp(&1.0) {
                            Ordering::Less => ValueCategory::Underfull,
                            Ordering::Equal => ValueCategory::Done,
                            Ordering::Greater => ValueCategory::Overfull,
                        };
                        (k, indexes[&token], None::<u16>, cat)
                    })
                    .collect::<Vec<_>>();
                counts.sort_by(|(k1, ..), (k2, ..)| k1.total_cmp(k2));

                'iter: loop {
                    let mut i = 0;
                    let mut j = buckets - 1;
                    while counts[i].3 != ValueCategory::Underfull {
                        if i >= buckets - 1 {
                            break 'iter;
                        }
                        i += 1;
                    }
                    while counts[j].3 != ValueCategory::Overfull {
                        if j == 0 {
                            break 'iter;
                        }
                        j -= 1;
                    }
                    counts[j].0 -= 1.0 - counts[i].0;
                    counts[i].2 = Some(counts[j].1);
                    counts[i].3 = ValueCategory::Done;
                    match counts[j].0.total_cmp(&1.0) {
                        Ordering::Less => {
                            counts[j].3 = ValueCategory::Underfull;
                        }
                        Ordering::Equal => {
                            counts[j].3 = ValueCategory::Done;
                        }
                        Ordering::Greater => {}
                    }
                }

                for (k, .., cat) in &mut counts {
                    if *cat != ValueCategory::Done {
                        *k = 1.0;
                        *cat = ValueCategory::Done;
                    }
                }

                for (k, i, j, ..) in counts {
                    let j = j.unwrap_or(i);
                    table += &format!("({k}_f64, {i}_u16, {j}_u16), ",);
                }

                table += "\n\t] },\n";
                Ok(table)
            })
            .collect::<Result<String, Error>>()?;

		let start_idx = indexes[&TokenType::Start];
		let outdir = outdir.join("weights.rs");
		fs::write(outdir, format!("const START_STATE: usize = {start_idx};\nconst MARKOV: &[crate::Token] = &[\n{aliases}];"))?;

		Ok(())
	}
}
