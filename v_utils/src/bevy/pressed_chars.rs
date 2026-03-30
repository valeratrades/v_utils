use std::collections::HashSet;

use bevy::{
	ecs::message::MessageReader,
	input::{
		ButtonState,
		keyboard::{Key, KeyboardInput},
	},
	prelude::*,
};

/// Layout-aware character input collected each frame from [`KeyboardInput`] events.
///
/// Insert as a resource and add [`update_pressed_chars`] as a system that runs before
/// any system that needs to read it.
#[derive(Default, Resource)]
pub struct PressedChars {
	/// Characters held down this frame.
	pub pressed: HashSet<char>,
	/// Characters that transitioned to pressed this frame (no repeats).
	pub just_pressed: HashSet<char>,
	/// Logical [`KeyCode`]s held down (respects system-level remapping).
	pub logical_keys_pressed: HashSet<KeyCode>,
	/// Logical [`KeyCode`]s that transitioned to pressed this frame.
	pub logical_keys_just_pressed: HashSet<KeyCode>,
}

/// System that collects [`KeyboardInput`] events into [`PressedChars`].
///
/// Schedule this *before* any system that reads `PressedChars`.
pub fn update_pressed_chars(mut keyboard_events: MessageReader<KeyboardInput>, mut pressed_chars: ResMut<PressedChars>) {
	pressed_chars.just_pressed.clear();
	pressed_chars.logical_keys_just_pressed.clear();

	for event in keyboard_events.read() {
		if let Key::Character(ref c) = event.logical_key {
			let ch = c.chars().next().unwrap_or('\0');
			match event.state {
				ButtonState::Pressed => {
					if !pressed_chars.pressed.contains(&ch) {
						pressed_chars.just_pressed.insert(ch);
					}
					pressed_chars.pressed.insert(ch);
				}
				ButtonState::Released => {
					pressed_chars.pressed.remove(&ch);
				}
			}
		}

		if let Some(keycode) = logical_key_to_keycode(&event.logical_key) {
			match event.state {
				ButtonState::Pressed => {
					if !pressed_chars.logical_keys_pressed.contains(&keycode) {
						pressed_chars.logical_keys_just_pressed.insert(keycode);
					}
					pressed_chars.logical_keys_pressed.insert(keycode);
				}
				ButtonState::Released => {
					pressed_chars.logical_keys_pressed.remove(&keycode);
				}
			}
		}
	}
}

fn logical_key_to_keycode(key: &Key) -> Option<KeyCode> {
	match key {
		Key::Escape => Some(KeyCode::Escape),
		Key::Enter => Some(KeyCode::Enter),
		Key::Tab => Some(KeyCode::Tab),
		Key::Space => Some(KeyCode::Space),
		Key::Backspace => Some(KeyCode::Backspace),
		Key::Delete => Some(KeyCode::Delete),
		Key::ArrowUp => Some(KeyCode::ArrowUp),
		Key::ArrowDown => Some(KeyCode::ArrowDown),
		Key::ArrowLeft => Some(KeyCode::ArrowLeft),
		Key::ArrowRight => Some(KeyCode::ArrowRight),
		Key::Home => Some(KeyCode::Home),
		Key::End => Some(KeyCode::End),
		Key::PageUp => Some(KeyCode::PageUp),
		Key::PageDown => Some(KeyCode::PageDown),
		Key::Shift => Some(KeyCode::ShiftLeft),
		Key::Control => Some(KeyCode::ControlLeft),
		Key::Alt => Some(KeyCode::AltLeft),
		_ => None,
	}
}
