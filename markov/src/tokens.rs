use std::{collections::HashMap, sync::Mutex};

use regex::Regex;
use string_interner::{DefaultBackend, StringInterner};

use crate::markov::TokenType;

pub type Tokens = HashMap<TokenType, HashMap<TokenType, usize>>;

lazy_static::lazy_static! {
	pub static ref INTERN: Mutex<StringInterner<DefaultBackend>> = Mutex::new(StringInterner::default());
	static ref TOKEN: Regex = Regex::new(r#"
        (?x)                                                    # ignore whitespace in regex
        (?<whitespace> \s+)?                                    # whitespace before tag
        <(?<close>/)?                                           # throw away closing tags
          (?<tag>       [[:alnum:]_-]+)
          (?<attrs>     (?:\s[[:alnum:]_-]+(?:=\\"[^"]*\\")?)*)
          (?<autoclose> /)?>                                    # detect if autoclosing tag
        | (?<html>      &[\w\#]+;                               # html sanitased symbols
        | (?<word>)
            \s*[\w’]+                                           # match words with spaces
          | \s*[[:punct:]&&[^<>]])                              # match punctuation with spaces
        "#.trim()).unwrap();
}

macro_rules! insert {
	($tokens:ident, $prev_token:ident, $token:expr) => {
		match $tokens.get_mut(&$prev_token) {
			Some(tokens) => {
				match tokens.get_mut(&$token) {
					Some(count) => *count += 1,
					None => {
						tokens.insert($token, 1);
					},
				};
			},
			None => {
				let mut map = HashMap::with_capacity(
					$crate::TOKEN_SUB_CAPACITY,
				);
				map.insert($token, 1);
				$tokens.insert($prev_token, map);
			},
		};
		$prev_token = $token;
	};
}

#[allow(unused_assignments)]
pub fn tokenise(post: &str, tokens: &Mutex<Tokens>) {
	let mut post = post;
	let mut tokens = tokens.lock().unwrap();
	let mut intern = INTERN.lock().unwrap();

	let mut prev_token = TokenType::Start;
	while let Some(cap) = TOKEN.captures(post) {
		post = &post[cap.get(0).unwrap().end()..];
		if cap.name("close").is_some() {
			insert!(tokens, prev_token, TokenType::End);
		} else if let Some(tag) = cap.name("tag") {
			let attrs = match cap.name("attrs") {
				Some(attrs) => intern.get_or_intern(attrs.as_str()),
				None => intern.get_or_intern_static(""),
			};

			let whitespace = cap
				.name("whitespace")
				.map(|w| intern.get_or_intern(w.as_str()));
			let tag = intern.get_or_intern(tag.as_str());
			insert!(
				tokens,
				prev_token,
				TokenType::Tag {
					whitespace,
					tag,
					attrs,
				}
			);

			if cap.name("autoclose").is_some() {
				insert!(tokens, prev_token, TokenType::End);
			}
		} else if let Some(html) = cap.name("html") {
			let html =
				html_escape::decode_html_entities(html.as_str());
			insert!(
				tokens,
				prev_token,
				TokenType::Text(intern.get_or_intern(&html))
			);
		} else if let Some(word) = cap.name("word") {
			insert!(
				tokens,
				prev_token,
				TokenType::Text(intern.get_or_intern(word.as_str()))
			);
		}
	}
	insert!(tokens, prev_token, TokenType::End);
}
