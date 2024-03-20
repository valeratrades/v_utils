use v_utils::llm;

fn main() {
	let mut conv = llm::Conversation::new();
	conv.add(llm::Role::User, "What is the cost of a haiku?");
	let response = llm::conversation(&conv, llm::Model::Fast, Some(10), Some(vec![" "])).unwrap();
	println!("{:?}", response);
}
