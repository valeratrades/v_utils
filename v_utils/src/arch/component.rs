// Lifecycle state machine for system components.
// Originally from nautilus_trader::common::component.

#![allow(unsafe_code)]

use std::{
	cell::{RefCell, UnsafeCell},
	collections::{HashMap, HashSet},
	fmt::{self, Debug},
	rc::Rc,
};

/// Components have state and lifecycle management capabilities.
pub trait Component: Debug {
	/// Returns the unique identifier for this component.
	fn component_id(&self) -> &str;

	fn state(&self) -> ComponentState;

	/// Transition the component with the state trigger.
	///
	/// # Panics
	///
	/// Panics if `trigger` is invalid for the current state.
	fn transition_state(&mut self, trigger: ComponentTrigger);

	fn is_ready(&self) -> bool {
		self.state() == ComponentState::Ready
	}

	fn is_running(&self) -> bool {
		self.state() == ComponentState::Running
	}

	fn is_stopped(&self) -> bool {
		self.state() == ComponentState::Stopped
	}

	fn is_degraded(&self) -> bool {
		self.state() == ComponentState::Degraded
	}

	fn is_faulted(&self) -> bool {
		self.state() == ComponentState::Faulted
	}

	fn is_disposed(&self) -> bool {
		self.state() == ComponentState::Disposed
	}

	fn initialize(&mut self) {
		self.transition_state(ComponentTrigger::Initialize);
	}

	fn start(&mut self) -> eyre::Result<()> {
		self.transition_state(ComponentTrigger::Start);

		if let Err(e) = self.on_start() {
			tracing::error!("{e}");
			return Err(e);
		}

		self.transition_state(ComponentTrigger::StartCompleted);
		Ok(())
	}

	fn stop(&mut self) -> eyre::Result<()> {
		self.transition_state(ComponentTrigger::Stop);

		if let Err(e) = self.on_stop() {
			tracing::error!("{e}");
			return Err(e);
		}

		self.transition_state(ComponentTrigger::StopCompleted);
		Ok(())
	}

	fn resume(&mut self) -> eyre::Result<()> {
		self.transition_state(ComponentTrigger::Resume);

		if let Err(e) = self.on_resume() {
			tracing::error!("{e}");
			return Err(e);
		}

		self.transition_state(ComponentTrigger::ResumeCompleted);
		Ok(())
	}

	fn reset(&mut self) -> eyre::Result<()> {
		self.transition_state(ComponentTrigger::Reset);

		if let Err(e) = self.on_reset() {
			tracing::error!("{e}");
			return Err(e);
		}

		self.transition_state(ComponentTrigger::ResetCompleted);
		Ok(())
	}

	fn degrade(&mut self) -> eyre::Result<()> {
		self.transition_state(ComponentTrigger::Degrade);

		if let Err(e) = self.on_degrade() {
			tracing::error!("{e}");
			return Err(e);
		}

		self.transition_state(ComponentTrigger::DegradeCompleted);
		Ok(())
	}

	fn fault(&mut self) -> eyre::Result<()> {
		self.transition_state(ComponentTrigger::Fault);

		if let Err(e) = self.on_fault() {
			tracing::error!("{e}");
			return Err(e);
		}

		self.transition_state(ComponentTrigger::FaultCompleted);
		Ok(())
	}

	fn dispose(&mut self) -> eyre::Result<()> {
		self.transition_state(ComponentTrigger::Dispose);

		if let Err(e) = self.on_dispose() {
			tracing::error!("{e}");
			return Err(e);
		}

		self.transition_state(ComponentTrigger::DisposeCompleted);
		Ok(())
	}

	fn on_start(&mut self) -> eyre::Result<()> {
		Ok(())
	}
	fn on_stop(&mut self) -> eyre::Result<()> {
		Ok(())
	}
	fn on_resume(&mut self) -> eyre::Result<()> {
		Ok(())
	}
	fn on_reset(&mut self) -> eyre::Result<()> {
		Ok(())
	}
	fn on_degrade(&mut self) -> eyre::Result<()> {
		Ok(())
	}
	fn on_fault(&mut self) -> eyre::Result<()> {
		Ok(())
	}
	fn on_dispose(&mut self) -> eyre::Result<()> {
		Ok(())
	}
}

/// The state of a component within the system.
#[derive(Clone, Copy, Debug, Default, derive_more::Display, Eq, Hash, PartialEq)]
pub enum ComponentState {
	/// When a component is instantiated, but not yet ready to fulfill its specification.
	#[default]
	PreInitialized,
	/// When a component is able to be started.
	Ready,
	/// When a component is executing its actions on `start`.
	Starting,
	/// When a component is operating normally and can fulfill its specification.
	Running,
	/// When a component is executing its actions on `stop`.
	Stopping,
	/// When a component has successfully stopped.
	Stopped,
	/// When a component is started again after its initial start.
	Resuming,
	/// When a component is executing its actions on `reset`.
	Resetting,
	/// When a component is executing its actions on `dispose`.
	Disposing,
	/// When a component has successfully shut down and released all of its resources.
	Disposed,
	/// When a component is executing its actions on `degrade`.
	Degrading,
	/// When a component has successfully degraded and may not meet its full specification.
	Degraded,
	/// When a component is executing its actions on `fault`.
	Faulting,
	/// When a component has successfully shut down due to a detected fault.
	Faulted,
}

#[rustfmt::skip]
impl ComponentState {
	/// Transition the state machine with the given `trigger`.
	///
	/// # Panics
	///
	/// Panics if `trigger` is invalid for the current state.
	pub fn transition(&mut self, trigger: ComponentTrigger) -> Self {
		let new_state = match (&self, trigger) {
			(Self::PreInitialized, ComponentTrigger::Initialize) => Self::Ready,
			(Self::Ready, ComponentTrigger::Reset) => Self::Resetting,
			(Self::Ready, ComponentTrigger::Start) => Self::Starting,
			(Self::Ready, ComponentTrigger::Dispose) => Self::Disposing,
			(Self::Resetting, ComponentTrigger::ResetCompleted) => Self::Ready,
			(Self::Starting, ComponentTrigger::StartCompleted) => Self::Running,
			(Self::Starting, ComponentTrigger::Stop) => Self::Stopping,
			(Self::Starting, ComponentTrigger::Fault) => Self::Faulting,
			(Self::Running, ComponentTrigger::Stop) => Self::Stopping,
			(Self::Running, ComponentTrigger::Degrade) => Self::Degrading,
			(Self::Running, ComponentTrigger::Fault) => Self::Faulting,
			(Self::Resuming, ComponentTrigger::Stop) => Self::Stopping,
			(Self::Resuming, ComponentTrigger::ResumeCompleted) => Self::Running,
			(Self::Resuming, ComponentTrigger::Fault) => Self::Faulting,
			(Self::Stopping, ComponentTrigger::StopCompleted) => Self::Stopped,
			(Self::Stopping, ComponentTrigger::Fault) => Self::Faulting,
			(Self::Stopped, ComponentTrigger::Reset) => Self::Resetting,
			(Self::Stopped, ComponentTrigger::Resume) => Self::Resuming,
			(Self::Stopped, ComponentTrigger::Dispose) => Self::Disposing,
			(Self::Stopped, ComponentTrigger::Fault) => Self::Faulting,
			(Self::Degrading, ComponentTrigger::DegradeCompleted) => Self::Degraded,
			(Self::Degraded, ComponentTrigger::Resume) => Self::Resuming,
			(Self::Degraded, ComponentTrigger::Stop) => Self::Stopping,
			(Self::Degraded, ComponentTrigger::Fault) => Self::Faulting,
			(Self::Disposing, ComponentTrigger::DisposeCompleted) => Self::Disposed,
			(Self::Faulting, ComponentTrigger::FaultCompleted) => Self::Faulted,
			_ => panic!("Invalid state transition: {self} -> {trigger}"),
		};
		*self = new_state;
		new_state
	}
}

/// A trigger condition for a component state transition.
#[derive(Clone, Copy, Debug, derive_more::Display, Eq, Hash, PartialEq)]
pub enum ComponentTrigger {
	Initialize,
	Start,
	StartCompleted,
	Stop,
	StopCompleted,
	Resume,
	ResumeCompleted,
	Reset,
	ResetCompleted,
	Dispose,
	DisposeCompleted,
	Degrade,
	DegradeCompleted,
	Fault,
	FaultCompleted,
}

// ── Registry ──────────────────────────────────────────────────────────

thread_local! {
	static COMPONENT_REGISTRY: ComponentRegistry = ComponentRegistry::new();
}

/// Registry for storing components with runtime borrow tracking.
///
/// The registry tracks which components are currently mutably borrowed to prevent
/// multiple simultaneous mutable borrows (which would be undefined behavior).
pub struct ComponentRegistry {
	components: RefCell<HashMap<String, Rc<UnsafeCell<dyn Component>>>>,
	borrows: RefCell<HashSet<String>>,
}

impl Debug for ComponentRegistry {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let components_ref = self.components.borrow();
		let keys: Vec<&String> = components_ref.keys().collect();
		f.debug_struct("ComponentRegistry")
			.field("components", &keys)
			.field("active_borrows", &self.borrows.borrow().len())
			.finish()
	}
}

impl Default for ComponentRegistry {
	fn default() -> Self {
		Self::new()
	}
}

impl ComponentRegistry {
	pub fn new() -> Self {
		Self {
			components: RefCell::new(HashMap::new()),
			borrows: RefCell::new(HashSet::new()),
		}
	}

	pub fn insert(&self, id: String, component: Rc<UnsafeCell<dyn Component>>) {
		self.components.borrow_mut().insert(id, component);
	}

	pub fn get(&self, id: &str) -> Option<Rc<UnsafeCell<dyn Component>>> {
		self.components.borrow().get(id).cloned()
	}

	/// Checks if a component is currently borrowed.
	pub fn is_borrowed(&self, id: &str) -> bool {
		self.borrows.borrow().contains(id)
	}

	/// Marks a component as borrowed. Returns false if already borrowed.
	fn try_borrow(&self, id: &str) -> bool {
		let mut borrows = self.borrows.borrow_mut();
		if borrows.contains(id) {
			false
		} else {
			borrows.insert(id.to_owned());
			true
		}
	}

	/// Releases a borrow on a component.
	fn release_borrow(&self, id: &str) {
		self.borrows.borrow_mut().remove(id);
	}
}

/// Guard that releases a component borrow when dropped.
///
/// This ensures borrows are released even if the code panics during
/// a lifecycle method call.
struct BorrowGuard {
	id: String,
}

impl BorrowGuard {
	fn new(id: String) -> Self {
		Self { id }
	}
}

impl Drop for BorrowGuard {
	fn drop(&mut self) {
		get_component_registry().release_borrow(&self.id);
	}
}

/// Returns a reference to the global component registry.
pub fn get_component_registry() -> &'static ComponentRegistry {
	COMPONENT_REGISTRY.with(|registry|
		// SAFETY: We return a static reference that lives for the lifetime of the thread.
		// Since this is thread_local storage, each thread has its own instance.
		unsafe { std::mem::transmute::<&ComponentRegistry, &'static ComponentRegistry>(registry) })
}

/// Registers a component in the global registry.
pub fn register_component<T>(component: T) -> Rc<UnsafeCell<T>>
where
	T: Component + 'static, {
	let component_id = component.component_id().to_owned();
	let component_ref = Rc::new(UnsafeCell::new(component));

	let component_trait_ref: Rc<UnsafeCell<dyn Component>> = component_ref.clone();
	get_component_registry().insert(component_id, component_trait_ref);

	component_ref
}

/// Returns a component from the global registry by ID.
pub fn get_component(id: &str) -> Option<Rc<UnsafeCell<dyn Component>>> {
	get_component_registry().get(id)
}

macro_rules! registry_lifecycle_fn {
	($fn_name:ident, $method:ident, $action:expr) => {
		/// Safely calls
		#[doc = $action]
		/// on a component in the global registry.
		///
		/// # Panics
		///
		/// Panics if the component is not found or is already borrowed.
		pub fn $fn_name(id: &str) -> eyre::Result<()> {
			let registry = get_component_registry();
			let component_ref = registry.get(id).unwrap_or_else(|| panic!("Component '{id}' not found in global registry"));

			assert!(registry.try_borrow(id), "Component '{id}' is already mutably borrowed — aliasing mutable references is UB",);

			let _guard = BorrowGuard::new(id.to_owned());

			// SAFETY: Borrow tracking ensures exclusive access
			unsafe {
				let component = &mut *component_ref.get();
				component.$method()
			}
		}
	};
}

registry_lifecycle_fn!(start_component, start, "start()");
registry_lifecycle_fn!(stop_component, stop, "stop()");
registry_lifecycle_fn!(reset_component, reset, "reset()");
registry_lifecycle_fn!(dispose_component, dispose, "dispose()");

#[cfg(test)]
/// Clears the component registry (for test isolation).
pub fn clear_component_registry() {
	let registry = get_component_registry();
	registry.components.borrow_mut().clear();
	registry.borrows.borrow_mut().clear();
}

#[cfg(test)]
mod tests {
	use std::sync::atomic::{AtomicBool, Ordering};

	use super::*;

	#[derive(Debug)]
	struct TestComponent {
		id: String,
		state: ComponentState,
		should_panic: &'static AtomicBool,
	}

	impl TestComponent {
		fn new(name: &str, should_panic: &'static AtomicBool) -> Self {
			Self {
				id: name.to_owned(),
				state: ComponentState::Ready,
				should_panic,
			}
		}
	}

	impl Component for TestComponent {
		fn component_id(&self) -> &str {
			&self.id
		}

		fn state(&self) -> ComponentState {
			self.state
		}

		fn transition_state(&mut self, trigger: ComponentTrigger) {
			self.state.transition(trigger);
		}

		#[allow(clippy::panic_in_result_fn)]
		fn on_start(&mut self) -> eyre::Result<()> {
			if self.should_panic.load(Ordering::SeqCst) {
				panic!("Intentional panic for testing");
			}
			Ok(())
		}
	}

	static NO_PANIC: AtomicBool = AtomicBool::new(false);
	static DO_PANIC: AtomicBool = AtomicBool::new(true);

	#[test]
	fn borrow_tracking_prevents_double_borrow() {
		clear_component_registry();

		let component = TestComponent::new("test-1", &NO_PANIC);
		let id = component.id.clone();

		let component_ref = Rc::new(UnsafeCell::new(component));
		get_component_registry().insert(id.clone(), component_ref);

		let result1 = start_component(&id);
		assert!(result1.is_ok());

		// Component should now be borrowable again (guard released)
		let result2 = stop_component(&id);
		assert!(result2.is_ok());
	}

	#[test]
	fn borrow_released_after_lifecycle_call() {
		clear_component_registry();

		let component = TestComponent::new("test-2", &NO_PANIC);
		let id = component.id.clone();

		let component_ref = Rc::new(UnsafeCell::new(component));
		get_component_registry().insert(id.clone(), component_ref);

		let _ = start_component(&id);
		assert!(!get_component_registry().is_borrowed(&id));
	}

	#[test]
	fn borrow_released_on_panic() {
		clear_component_registry();

		let component = TestComponent::new("test-panic", &DO_PANIC);
		let id = component.id.clone();

		let component_ref = Rc::new(UnsafeCell::new(component));
		get_component_registry().insert(id.clone(), component_ref);

		let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
			let _ = start_component(&id);
		}));
		assert!(result.is_err(), "Expected panic from on_start");

		assert!(!get_component_registry().is_borrowed(&id), "Borrow was not released after panic");
	}
}
