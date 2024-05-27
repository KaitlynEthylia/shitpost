#[cfg(not(feature = "_"))]
type InternedString = &'static str;
#[cfg(feature = "_")]
type InternedString = string_interner::symbol::SymbolU32;

#[allow(dead_code)]
type AliasTable = &'static [(f64, u16, u16)];

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum TokenType {
	Start,
	Text(InternedString),
	Tag {
		whitespace: Option<InternedString>,
		tag: InternedString,
		attrs: InternedString,
	},
	End,
}

#[derive(Debug)]
#[allow(dead_code)]
struct Token {
	token: TokenType,
	alias: AliasTable,
}

#[cfg(not(feature = "_"))]
mod markov {
	use std::ops::Shr;

	use rand::Rng;

	include!(concat!(env!("OUT_DIR"), "/", "weights.rs"));

	fn idx_and_y(rand: &mut impl Rng) -> (u16, f64) {
		let a: u64 = rand.gen();
		let b: f64 =
			f64::from_bits((a | 0xFFF0 << 48) & !(0xC << 60));
		let a =
			(a.shr(48) as u16 & 0xFF00) + b.to_bits() as u8 as u16;
		(a, b - 1_f64)
	}

	pub fn generate() -> String {
		let mut state = &MARKOV[START_STATE];
		let mut rand = rand::thread_rng();

		let mut output = String::with_capacity(256);
		let mut scope = Vec::with_capacity(4);

		loop {
			match state.token {
				super::TokenType::Text(s) => output += s,
				// ignore attrs so mentions don't become actual links, might do something better to
				// specifically filter out mentions later
				super::TokenType::Tag {
					whitespace, tag, ..
				} if scope.len() < 3 => {
					scope.push(tag);
					if let Some(whitespace) = whitespace {
						output += whitespace;
					}
					output.push('<');
					output += tag;
					output.push('>')
				},
				super::TokenType::End => match scope.pop() {
					Some(tag) => {
						output += "</";
						output += tag;
						output += ">";
					},
					None => break,
				},
				_ => {},
			}
			let (idx, y) = idx_and_y(&mut rand);
			let idx = idx as usize % state.alias.len();
			let (p, token, alt) = state.alias[idx];
			state = &MARKOV
				[if y <= p { token as usize } else { alt as usize }];
		}

		output
	}
}
