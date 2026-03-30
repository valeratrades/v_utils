use std::collections::HashMap;

use bevy::{ecs::message::MessageWriter, prelude::*};

use super::PressedChars;

/// Default timeout before showing hints popup (seconds).
pub const MODAL_HINT_TIMEOUT: f32 = 0.150;

/// A node in a modal keybind tree, generic over action type.
#[derive(Clone, Debug, Default)]
pub struct ModalNode<A> {
	/// Child nodes keyed by character.
	pub children: HashMap<char, ModalNode<A>>,
	/// Action to execute if this is a terminal node.
	pub action: Option<A>,
	/// Human-readable label for this key (shown in hints).
	pub label: Option<&'static str>,
}

impl<A: Clone> ModalNode<A> {
	pub fn new() -> Self {
		Self {
			children: HashMap::new(),
			action: None,
			label: None,
		}
	}

	/// Add a child node for a key.
	pub fn child(mut self, key: char, label: &'static str, node: ModalNode<A>) -> Self {
		let mut child = node;
		child.label = Some(label);
		self.children.insert(key, child);
		self
	}

	/// Create a terminal node with an action.
	pub fn action(action: A) -> Self {
		Self {
			action: Some(action),
			children: HashMap::new(),
			label: None,
		}
	}

	pub fn has_children(&self) -> bool {
		!self.children.is_empty()
	}

	pub fn is_terminal(&self) -> bool {
		self.action.is_some()
	}
}

/// Resource tracking current modal input state, generic over action type.
#[derive(Resource)]
pub struct ModalState<A> {
	/// Root of the keybind tree.
	pub root: ModalNode<A>,
	/// Current sequence of keys pressed.
	pub sequence: Vec<char>,
	/// Time since last key in sequence.
	pub time_since_last_key: f32,
	/// Whether hints popup is visible.
	pub hints_visible: bool,
	/// Currently in a modal sequence.
	pub active: bool,
	/// Showing full help.
	pub show_help: bool,
}

impl<A: Clone> Default for ModalState<A> {
	fn default() -> Self {
		Self {
			root: ModalNode::new(),
			sequence: Vec::new(),
			time_since_last_key: 0.0,
			hints_visible: false,
			active: false,
			show_help: false,
		}
	}
}

impl<A: Clone> ModalState<A> {
	pub fn new(root: ModalNode<A>) -> Self {
		Self { root, ..Default::default() }
	}

	/// Get the current node based on the sequence.
	pub fn current_node(&self) -> Option<&ModalNode<A>> {
		let mut node = &self.root;
		for &key in &self.sequence {
			node = node.children.get(&key)?;
		}
		Some(node)
	}

	/// Reset the modal state.
	pub fn reset(&mut self) {
		self.sequence.clear();
		self.time_since_last_key = 0.0;
		self.hints_visible = false;
		self.active = false;
		self.show_help = false;
	}

	/// Process a key press. Returns action if a terminal node is reached.
	///
	/// If the current node has exactly one child matching `key` and that child
	/// is terminal, the action fires immediately. If the node itself has only
	/// one child total (auto-advance), we skip straight through without waiting
	/// for further input.
	pub fn process_key(&mut self, key: char) -> Option<A> {
		let (is_terminal, action) = {
			let current = self.current_node()?;
			let next_node = current.children.get(&key)?;
			(next_node.is_terminal(), next_node.action.clone())
		};

		self.sequence.push(key);
		self.time_since_last_key = 0.0;
		self.hints_visible = false;
		self.active = true;

		if is_terminal {
			self.reset();
			return action;
		}

		// Auto-advance through single-child branches
		self.try_auto_advance()
	}

	/// If the current node has exactly one child, advance through it automatically.
	/// Repeats until we hit a terminal or a branch with multiple children.
	fn try_auto_advance(&mut self) -> Option<A> {
		loop {
			let (key, is_terminal, action) = {
				let current = self.current_node()?;
				if current.children.len() != 1 {
					return None;
				}
				let (&k, child) = current.children.iter().next().unwrap();
				(k, child.is_terminal(), child.action.clone())
			};
			self.sequence.push(key);

			if is_terminal {
				self.reset();
				return action;
			}
		}
	}

	/// Check if a key would be valid at the current position.
	pub fn is_valid_key(&self, key: char) -> bool {
		if let Some(current) = self.current_node() {
			current.children.contains_key(&key)
		} else {
			self.root.children.contains_key(&key)
		}
	}
}

/// System that drives [`ModalState`]. Add to your update schedule after [`update_pressed_chars`].
///
/// Completed actions are written as [`ModalActionFired<A>`] events.
pub fn update_modal_state<A: Clone + Send + Sync + 'static>(
	time: Res<Time>,
	pressed_chars: Res<PressedChars>,
	mut modal_state: ResMut<ModalState<A>>,
	mut actions: MessageWriter<ModalActionFired<A>>,
) {
	// Escape resets
	if pressed_chars.logical_keys_just_pressed.contains(&KeyCode::Escape) && (modal_state.active || modal_state.show_help) {
		modal_state.reset();
		return;
	}

	// Dismiss help on any key, but let the key also start a new sequence
	if modal_state.show_help && !pressed_chars.just_pressed.is_empty() {
		modal_state.reset();
	}

	// Hint timeout
	if modal_state.active {
		modal_state.time_since_last_key += time.delta_secs();
		if modal_state.time_since_last_key >= MODAL_HINT_TIMEOUT && !modal_state.hints_visible {
			modal_state.hints_visible = true;
		}
	}

	// Process keys
	for &key in &pressed_chars.just_pressed {
		if modal_state.active {
			if modal_state.is_valid_key(key) {
				if let Some(action) = modal_state.process_key(key) {
					actions.write(ModalActionFired(action));
				}
			} else {
				modal_state.reset();
			}
		} else if modal_state.root.children.contains_key(&key) {
			if let Some(action) = modal_state.process_key(key) {
				actions.write(ModalActionFired(action));
			}
		}
	}
}

/// Event fired when a modal key sequence completes.
#[derive(Message)]
pub struct ModalActionFired<A>(pub A);
