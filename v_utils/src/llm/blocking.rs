use crate::llm::{self, Conversation, Model, Response};
use eyre::Result;
use tokio::runtime::Runtime;

pub fn oneshot<T: AsRef<str>>(message: T, model: Model) -> Result<Response> {
	let runtime = tokio::runtime::Runtime::new().unwrap();
	runtime.block_on(llm::oneshot(message, model))
}

pub fn conversation(conv: &Conversation, model: Model, max_tokens: Option<usize>, stop_sequences: Option<Vec<&str>>) -> Result<Response> {
	let runtime = tokio::runtime::Runtime::new().unwrap();
	runtime.block_on(llm::conversation(conv, model, max_tokens, stop_sequences))
}
