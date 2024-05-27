use std::{
	pin::Pin,
	sync::{Arc, Mutex},
	task::{Context, Poll},
};

use actson::{
	tokio::AsyncBufReaderJsonFeeder, JsonEvent, JsonParser,
};
use bytes::Bytes;
use futures::{Stream, StreamExt};
use reqwest::{Error as RequestError, RequestBuilder};
use tokio::{
	io::{
		AsyncRead, BufReader, Error as IOError, ReadBuf,
		Result as IOResult,
	},
	sync::mpsc::UnboundedSender,
};

use crate::{
	tokens::{self, Tokens},
	Error,
};

#[derive(Debug, Default)]
struct ParseState {
	state: State,
	id_found: bool,
}

#[derive(Debug, Default)]
enum State {
	#[default]
	Start,
	Depth(u8),
	ID,
	Content,
}

struct AsyncStreamReader<T> {
	stream: T,
	overflow: Option<Bytes>,
}

impl<T> AsyncRead for AsyncStreamReader<T>
where
	T: Stream<Item = Result<Bytes, RequestError>> + Unpin,
{
	fn poll_read(
		self: Pin<&mut Self>,
		cx: &mut Context<'_>,
		buf: &mut ReadBuf<'_>,
	) -> Poll<IOResult<()>> {
		let self_ = self.get_mut();
		let data = match self_.overflow.take() {
			Some(data) if !data.is_empty() => data,
			_ => match self_.stream.poll_next_unpin(cx) {
				Poll::Pending => return Poll::Pending,
				Poll::Ready(None) => return Poll::Ready(Ok(())),
				Poll::Ready(Some(data)) => {
					data.map_err(IOError::other)?
				},
			},
		};

		let rem = buf.remaining();
		#[allow(clippy::cast_possible_wrap)]
		match (data.len() as isize).wrapping_sub_unsigned(rem) {
			0.. => {
				buf.put_slice(&data[..rem]);
				self_.overflow = Some(data.slice(rem..));
			},
			_ => {
				buf.put_slice(&data);
			},
		};

		Poll::Ready(Ok(()))
	}
}

pub async fn process_posts(
	rq: RequestBuilder,
	min_id: String,
	send: UnboundedSender<Result<Option<String>, Error>>,
	tokens: Arc<Mutex<Tokens>>,
) {
	let stream = match rq.query(&[("min_id", &min_id)]).send().await {
		Ok(s) => s.bytes_stream(),
		Err(e) => {
			send.send(Err(Error::RequestError(e)))
				.expect("Sending channel closed unexpectedly.");
			return;
		},
	};
	let stream = AsyncStreamReader {
		stream,
		overflow: None,
	};
	let reader = BufReader::new(stream);
	let feeder = AsyncBufReaderJsonFeeder::new(reader);
	let mut parser = JsonParser::new(feeder);

	let mut state = ParseState::default();

	let mut parse_result = parser.next_event();
	while let Ok(Some(event)) = parse_result {
		match (event, &mut state.state) {
			(JsonEvent::NeedMoreInput, _) => {
				if let Err(e) = parser.feeder.fill_buf().await {
					send.send(Err(e.into())).expect(
						"Sending channel closed unexpectedly.",
					);
				}
			},
			(JsonEvent::StartObject, State::Start) => {
				state.state = State::Depth(1)
			},
			(JsonEvent::StartObject, State::Depth(ref mut depth)) => {
				*depth += 1
			},
			(JsonEvent::EndObject, State::Depth(ref mut depth)) => {
				*depth -= 1
			},
			(JsonEvent::FieldName, State::Depth(1)) => {
				match parser.current_str() {
					Ok("content") => state.state = State::Content,
					Ok("id") if !state.id_found => {
						state.state = State::ID
					},
					_ => {},
				}
			},
			(JsonEvent::ValueString, State::ID) => {
				match parser.current_str() {
					Ok(id) => {
						send.send(Ok(Some(id.to_string()))).expect(
							"Sending channel closed unexpectedly.",
						);
						state = ParseState {
							id_found: true,
							state: State::Depth(1),
						};
					},
					Err(e) => send.send(Err(e.into())).expect(
						"Sending channel closed unexpectedly.",
					),
				}
			},
			(JsonEvent::ValueString, State::Content) => {
				match parser.current_str() {
					Ok(content) => {
						tokens::tokenise(content, &tokens);
						state.state = State::Depth(1);
					},
					Err(e) => send.send(Err(e.into())).expect(
						"Sending channel closed unexpectedly.",
					),
				}
			},
			_ => {},
		}
		parse_result = parser.next_event();
	}
	if let Err(e) = parse_result {
		send.send(Err(e.into()))
			.expect("Sending channel closed unexpectedly.");
	}

	if let State::Start = state.state {
		send.send(Ok(None))
			.expect("Sending channel closed unexpectedly.");
	}
}
