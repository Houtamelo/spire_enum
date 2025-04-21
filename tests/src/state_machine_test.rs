use std::{fmt::Debug, marker::PhantomData};

use super::*;

// States for our door state machine
#[derive(Clone, Debug)]
pub struct Open;

#[derive(Clone, Debug)]
pub struct Closed;

#[derive(Clone, Debug)]
pub struct Locked;

// Door that uses typestate pattern
#[derive(Clone, Debug)]
pub struct Door<State> {
    name: String,
    state: PhantomData<State>,
}

// Implementations for each state
impl Door<Open> {
    pub fn new(name: impl Into<String>) -> Self {
        Door {
            name: name.into(),
            state: PhantomData,
        }
    }

    pub fn close(self) -> Door<Closed> {
        println!("Door '{}' is being closed", self.name);
        Door {
            name: self.name,
            state: PhantomData,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Door<Closed> {
    pub fn open(self) -> Door<Open> {
        println!("Door '{}' is being opened", self.name);
        Door {
            name: self.name,
            state: PhantomData,
        }
    }

    pub fn lock(self) -> Door<Locked> {
        println!("Door '{}' is being locked", self.name);
        Door {
            name: self.name,
            state: PhantomData,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Door<Locked> {
    pub fn unlock(self) -> Door<Closed> {
        println!("Door '{}' is being unlocked", self.name);
        Door {
            name: self.name,
            state: PhantomData,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

// Door action enum - no delegated_enum needed here
#[derive(Clone, Debug)]
pub enum DoorAction {
    OpenDoor,
    CloseDoor,
    LockDoor,
    UnlockDoor,
}

// Implementation of DoorAction without using delegate_impl
impl DoorAction {
    pub fn apply_to_open_door(self, door: Door<Open>) -> Result<DoorState, &'static str> {
        match self {
            DoorAction::CloseDoor => Ok(DoorState::Closed(door.close())),
            DoorAction::OpenDoor => Err("Door is already open"),
            DoorAction::LockDoor => Err("Cannot lock an open door"),
            DoorAction::UnlockDoor => Err("Cannot unlock an open door"),
        }
    }

    pub fn apply_to_closed_door(self, door: Door<Closed>) -> Result<DoorState, &'static str> {
        match self {
            DoorAction::OpenDoor => Ok(DoorState::Open(door.open())),
            DoorAction::CloseDoor => Err("Door is already closed"),
            DoorAction::LockDoor => Ok(DoorState::Locked(door.lock())),
            DoorAction::UnlockDoor => Err("Door is not locked"),
        }
    }

    pub fn apply_to_locked_door(self, door: Door<Locked>) -> Result<DoorState, &'static str> {
        match self {
            DoorAction::OpenDoor => Err("Cannot open a locked door"),
            DoorAction::CloseDoor => Err("Door is already closed"),
            DoorAction::LockDoor => Err("Door is already locked"),
            DoorAction::UnlockDoor => Ok(DoorState::Closed(door.unlock())),
        }
    }
}

// Door state using delegated_enum macro correctly
#[delegated_enum]
#[derive(Clone)]
pub enum DoorState {
    Open(Door<Open>),
    Closed(Door<Closed>),
    Locked(Door<Locked>),
}

// Regular implementation for DoorState (not using delegate_impl)
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
#[derive(Clone)]
pub struct DoorManager {
    state: DoorState,
}

impl DoorManager {
    pub fn new(state: DoorState) -> Self {
        Self { state }
    }

    pub fn state(&self) -> &DoorState {
        &self.state
    }

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
            .apply_action(DoorAction::CloseDoor)
            .expect("Should close successfully");
        assert_eq!(state.state_name(), "closed");

        let result = state.clone().apply_action(DoorAction::CloseDoor);
        assert!(result.is_err(), "Closing a closed door should fail");

        let state = state
            .apply_action(DoorAction::LockDoor)
            .expect("Should lock successfully");
        assert_eq!(state.state_name(), "locked");

        let result = state.clone().apply_action(DoorAction::OpenDoor);
        assert!(result.is_err(), "Opening a locked door should fail");

        let state = state
            .apply_action(DoorAction::UnlockDoor)
            .expect("Should unlock successfully");
        assert_eq!(state.state_name(), "closed");

        let state = state
            .apply_action(DoorAction::OpenDoor)
            .expect("Should open successfully");
        assert_eq!(state.state_name(), "open");
    }

    #[test]
    fn test_door_state_transitions() {
        let door = Door::new("Back Door");
        let mut state = DoorState::Open(door);

        // Define a sequence of actions
        let actions = vec![
            DoorAction::CloseDoor,
            DoorAction::LockDoor,
            DoorAction::UnlockDoor,
            DoorAction::OpenDoor,
            DoorAction::CloseDoor,
        ];

        // Expected states after each action
        let expected_states = vec!["closed", "locked", "closed", "open", "closed"];

        // Apply actions and verify states
        for (i, action) in actions.into_iter().enumerate() {
            state = state
                .apply_action(action)
                .expect(&format!("Action {} should succeed", i));
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
            .transition(DoorAction::CloseDoor)
            .expect("Should close successfully");
        assert_eq!(manager.state().state_name(), "closed");

        manager = manager
            .transition(DoorAction::LockDoor)
            .expect("Should lock successfully");
        assert_eq!(manager.state().state_name(), "locked");
    }

    #[test]
    fn test_delegated_macro() {
        let door = Door::new("Delegated Door");
        let state = DoorState::Open(door);

        // Test using the delegated macro directly
        let name = delegate_door_state!(state => |door| door.name().to_string());
        assert_eq!(name, "Delegated Door");
    }
}
