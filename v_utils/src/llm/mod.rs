use anyhow::Result;
use serde::Serialize;

mod claude;

pub fn oneshot<T: AsRef<str>>(message: T, model: Model) -> Result<Response> {
	let mut conv = Conversation::new("");
	conv.add(Role::User, message);
	conversation(&conv, model)
}

pub fn conversation(conv: &Conversation, model: Model) -> Result<Response> {
	claude::ask_claude(conv, model)
}

#[derive(Clone, Debug)]
pub enum Model {
	Fast,
	Medium,
	Slow,
}

pub enum Role {
	System,
	User,
	Assistant,
}
pub struct Message {
	role: Role,
	content: String,
}
impl Message {
	fn new<T: AsRef<str>>(role: Role, content: T) -> Self {
		Self {
			role,
			content: content.as_ref().to_string(),
		}
	}
}
pub struct Conversation {
	pub messages: Vec<Message>,
}
impl Conversation {
	pub fn new<T: AsRef<str>>(system_message: T) -> Self {
		Self {
			messages: vec![Message::new(Role::System, system_message)],
		}
	}

	pub fn add<T: AsRef<str>>(&mut self, role: Role, content: T) {
		self.messages.push(Message::new(role, content));
	}

	pub fn add_exchange<T: AsRef<str>>(&mut self, user_message: T, assistant_message: T) {
		self.add(Role::User, user_message);
		self.add(Role::Assistant, assistant_message);
	}
}

pub struct Response {
	pub text: String,
	pub cost_cents: f32,
}
impl std::fmt::Display for Response {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Response: {}\nCost (cents): {}", self.text, self.cost_cents)
	}
}
impl Response {
	pub fn extract_codeblocks(&self, extension: &str) -> Vec<String> {
		self.text
			.split("```")
			.filter_map(|s| {
				if s.starts_with(extension) {
					Some(s.strip_prefix(extension).unwrap().to_string())
				} else {
					None
				}
			})
			.collect()
	}
}

trait LlmResponse {
	fn to_general_form(&self) -> Response;
}

trait LlmConversation: Serialize {
	fn new(conversation: &Conversation) -> Self;
}
