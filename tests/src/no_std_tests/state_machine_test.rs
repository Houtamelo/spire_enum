use core::{fmt::Debug, marker::PhantomData};

use super::*;

// States for our door state machine
#[allow(unused)]
#[derive(Clone, Debug)]
pub struct Open;

#[allow(unused)]
#[derive(Clone, Debug)]
pub struct Closed;

#[allow(unused)]
#[derive(Clone, Debug)]
pub struct Locked;

// Door that uses typestate pattern
#[allow(unused)]
#[derive(Clone, Debug)]
pub struct Door<State> {
    name:  &'static str,
    state: PhantomData<State>,
}

#[allow(unused)]
impl Door<Open> {
    pub fn new(name: &'static str) -> Self {
        Door {
            name,
            state: PhantomData,
        }
    }

    pub fn close(self) -> Door<Closed> {
        Door {
            name:  self.name,
            state: PhantomData,
        }
    }

    pub fn name(&self) -> &'static str { self.name }
}

#[allow(unused)]
impl Door<Closed> {
    pub fn open(self) -> Door<Open> {
        Door {
            name:  self.name,
            state: PhantomData,
        }
    }

    pub fn lock(self) -> Door<Locked> {
        Door {
            name:  self.name,
            state: PhantomData,
        }
    }

    pub fn name(&self) -> &'static str { self.name }
}

#[allow(unused)]
impl Door<Locked> {
    pub fn unlock(self) -> Door<Closed> {
        Door {
            name:  self.name,
            state: PhantomData,
        }
    }

    pub fn name(&self) -> &'static str { self.name }
}

// Door action enum - no delegated_enum needed here
#[allow(unused)]
#[derive(Clone, Debug)]
pub enum DoorAction {
    Open,
    Close,
    Lock,
    Unlock,
}

// Implementation of DoorAction without using delegate_impl
#[allow(unused)]
impl DoorAction {
    pub fn apply_to_open_door(self, door: Door<Open>) -> Result<DoorState, &'static str> {
        match self {
            DoorAction::Close => Ok(DoorState::Closed(door.close())),
            DoorAction::Open => Err("Door is already open"),
            DoorAction::Lock => Err("Cannot lock an open door"),
            DoorAction::Unlock => Err("Cannot unlock an open door"),
        }
    }

    pub fn apply_to_closed_door(self, door: Door<Closed>) -> Result<DoorState, &'static str> {
        match self {
            DoorAction::Open => Ok(DoorState::Open(door.open())),
            DoorAction::Close => Err("Door is already closed"),
            DoorAction::Lock => Ok(DoorState::Locked(door.lock())),
            DoorAction::Unlock => Err("Door is not locked"),
        }
    }

    pub fn apply_to_locked_door(self, door: Door<Locked>) -> Result<DoorState, &'static str> {
        match self {
            DoorAction::Open => Err("Cannot open a locked door"),
            DoorAction::Close => Err("Door is already closed"),
            DoorAction::Lock => Err("Door is already locked"),
            DoorAction::Unlock => Ok(DoorState::Closed(door.unlock())),
        }
    }
}

// Door state using delegated_enum macro correctly
#[delegated_enum]
#[allow(unused)]
#[derive(Clone)]
pub enum DoorState {
    Open(Door<Open>),
    Closed(Door<Closed>),
    Locked(Door<Locked>),
}

// Regular implementation for DoorState (not using delegate_impl)
#[allow(unused)]
impl DoorState {
    pub fn apply_action(self, action: DoorAction) -> Result<DoorState, &'static str> {
        match self {
            DoorState::Open(door) => action.apply_to_open_door(door),
            DoorState::Closed(door) => action.apply_to_closed_door(door),
            DoorState::Locked(door) => action.apply_to_locked_door(door),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            DoorState::Open(door) => door.name(),
            DoorState::Closed(door) => door.name(),
            DoorState::Locked(door) => door.name(),
        }
    }

    pub fn state_name(&self) -> &'static str {
        match self {
            DoorState::Open(_) => "open",
            DoorState::Closed(_) => "closed",
            DoorState::Locked(_) => "locked",
        }
    }
}

// Example of proper delegation pattern
#[allow(unused)]
#[derive(Clone)]
pub struct DoorManager {
    state: DoorState,
}

#[allow(unused)]
impl DoorManager {
    pub fn new(state: DoorState) -> Self { Self { state } }

    pub fn state(&self) -> &DoorState { &self.state }

    pub fn transition(mut self, action: DoorAction) -> Result<Self, &'static str> {
        self.state = self.state.apply_action(action)?;
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_door_state_machine() {
        // Create a new door in the Open state
        let door = Door::new("Front Door");
        let state = DoorState::Open(door);

        // Try valid and invalid operations
        let state = state
            .apply_action(DoorAction::Close)
            .expect("Should close successfully");
        assert_eq!(state.state_name(), "closed");

        let result = state.clone().apply_action(DoorAction::Close);
        assert!(result.is_err(), "Closing a closed door should fail");

        let state = state
            .apply_action(DoorAction::Lock)
            .expect("Should lock successfully");
        assert_eq!(state.state_name(), "locked");

        let result = state.clone().apply_action(DoorAction::Open);
        assert!(result.is_err(), "Opening a locked door should fail");

        let state = state
            .apply_action(DoorAction::Unlock)
            .expect("Should unlock successfully");
        assert_eq!(state.state_name(), "closed");

        let state = state
            .apply_action(DoorAction::Open)
            .expect("Should open successfully");
        assert_eq!(state.state_name(), "open");
    }

    #[test]
    fn test_door_state_transitions() {
        let door = Door::new("Back Door");
        let mut state = DoorState::Open(door);

        // Define a sequence of actions
        let actions = [
            DoorAction::Close,
            DoorAction::Lock,
            DoorAction::Unlock,
            DoorAction::Open,
            DoorAction::Close,
        ];

        // Expected states after each action
        let expected_states = ["closed", "locked", "closed", "open", "closed"];

        // Apply actions and verify states
        for (i, action) in actions.into_iter().enumerate() {
            state = state
                .apply_action(action)
                .unwrap_or_else(|_| panic!("Action {} should succeed", i));
            assert_eq!(state.state_name(), expected_states[i]);
        }
    }

    #[test]
    fn test_door_manager() {
        let door = Door::new("Manager Door");
        let state = DoorState::Open(door);
        let mut manager = DoorManager::new(state);

        // Test proper delegation
        assert_eq!(manager.state().state_name(), "open");

        // Test transitions
        manager = manager
            .transition(DoorAction::Close)
            .expect("Should close successfully");
        assert_eq!(manager.state().state_name(), "closed");

        manager = manager
            .transition(DoorAction::Lock)
            .expect("Should lock successfully");
        assert_eq!(manager.state().state_name(), "locked");
    }

    #[test]
    fn test_delegated_macro() {
        let door = Door::new("Delegated Door");
        let state = DoorState::Open(door);

        // Test using the delegated macro directly
        let name = delegate_door_state!(state => |door| door.name());
        assert_eq!(name, "Delegated Door");
    }
}
