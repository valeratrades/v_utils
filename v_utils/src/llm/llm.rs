use anyhow::Result;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::{Deserialize, Serialize, Serializer};
use serde_json::json;
use std::io::Write;

// Idea is to have this be updated to use the best model currently available, and generic input parameters

pub fn oneshot<T: AsRef<str>>(message: T, model: Model) -> Result<Response> {
	let mut conv = Conversation::new("");
	conv.add(Role::User, message);
	conversation(&conv, model)
}

pub fn conversation(conv: &Conversation, model: Model) -> Result<Response> {
	let claude_conv = ClaudeConversation::new(conv);
	let response = ask_claude(claude_conv, {
		match model {
			Model::Fast => ClaudeModel::Haiku,
			Model::Medium => ClaudeModel::Sonnet,
			Model::Slow => ClaudeModel::Opus,
		}
	})?;
	Ok(response.to_general_form())
}

#[derive(Clone, Debug)]
pub enum Model {
	Fast,
	Medium,
	Slow,
}

enum Role {
	System,
	User,
	Assistant,
}
struct Message {
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

//=============================================================================
// Specific implementations
//=============================================================================

#[allow(dead_code)]
#[derive(Debug)]
enum ClaudeModel {
	Haiku,
	Sonnet,
	Opus,
}
pub struct Cost {
	pub million_input_tokens: f32,
	pub million_output_tokens: f32,
}

impl ClaudeModel {
	fn to_str(&self) -> &str {
		match self {
			ClaudeModel::Haiku => "claude-3-haiku-20240307",
			ClaudeModel::Sonnet => "claude-3-sonnet-20240229",
			ClaudeModel::Opus => "claude-3-opus-20240229",
		}
	}

	fn from_str(s: &str) -> Self {
		match s {
			_ if s.to_lowercase().contains("haiku") => Self::Haiku,
			_ if s.to_lowercase().contains("sonnet") => Self::Sonnet,
			_ if s.to_lowercase().contains("opus") => Self::Opus,
			_ => panic!("Unknown model: {}", s),
		}
	}

	pub fn cost(&self) -> Cost {
		match self {
			Self::Haiku => Cost {
				million_input_tokens: 0.25,
				million_output_tokens: 1.25,
			},
			Self::Sonnet => Cost {
				million_input_tokens: 3.0,
				million_output_tokens: 15.0,
			},
			Self::Opus => Cost {
				million_input_tokens: 15.0,
				million_output_tokens: 75.0,
			},
		}
	}
}

#[derive(Debug, Serialize)]
struct ClaudeMessage {
	role: String,
	content: String,
}
#[derive(Debug, Serialize)]
struct ClaudeConversation {
	messages: Vec<ClaudeMessage>,
}
impl LlmConversation for ClaudeConversation {
	fn new(conversation: &Conversation) -> Self {
		let mut messages = Vec::new();
		for message in &conversation.messages {
			messages.push(ClaudeMessage {
				role: {
					match message.role {
						Role::System => "system".to_string(),
						Role::User => "user".to_string(),
						Role::Assistant => "assistant".to_string(),
					}
				},
				content: message.content.to_string(),
			});
		}
		Self { messages }
	}
}

///docs: https://docs.anthropic.com/claude/reference/messages_post
fn ask_claude(conversation: ClaudeConversation, model: ClaudeModel) -> Result<ClaudeResponse> {
	let api_key = std::env::var("CLAUDE_TOKEN").expect("CLAUDE_TOKEN environment variable not set");
	let url = "https://api.anthropic.com/v1/messages";

	let mut headers = HeaderMap::new();
	headers.insert("x-api-key", HeaderValue::from_str(&api_key).unwrap());
	headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));
	headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

	let payload = json!({
		"model": model.to_str(),
		"max_tokens": 1024,
		"messages": conversation
	});

	let client = Client::new();
	let response = client.post(url).headers(headers).json(&payload).send().expect("Failed to send request");

	let response_raw = response.text().expect("Failed to read response body");
	let response: ClaudeResponse = serde_json::from_str(&response_raw).expect("Failed to parse response body");
	Ok(response)
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct ClaudeResponse {
	id: String,
	#[serde(rename = "type")]
	response_type: String,
	role: String,
	content: Vec<ClaudeContent>,
	model: String,
	stop_reason: String,
	stop_sequence: Option<String>,
	usage: ClaudeUsage,
}

impl ClaudeResponse {
	pub fn text(&self) -> String {
		self.content[0].text.clone()
	}

	pub fn cost_cents(&self) -> f32 {
		let model = ClaudeModel::from_str(&self.model);
		let cost = model.cost();
		(self.usage.input_tokens as f32 * cost.million_input_tokens + self.usage.output_tokens as f32 * cost.million_output_tokens) / 10_000.0
	}
}

impl LlmResponse for ClaudeResponse {
	fn to_general_form(&self) -> Response {
		Response {
			text: self.text(),
			cost_cents: self.cost_cents(),
		}
	}
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct ClaudeContent {
	#[serde(rename = "type")]
	content_type: String,
	text: String,
}
#[derive(Deserialize, Debug)]
struct ClaudeUsage {
	input_tokens: u32,
	output_tokens: u32,
}
