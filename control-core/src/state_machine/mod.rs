//! State Machine module - XState compatible state machine interpreter
//!
//! This module provides a complete implementation of a state machine interpreter
//! compatible with XState JSON format. It supports:
//!
//! - States with entry/exit actions
//! - Transitions with guards and actions
//! - Final and compound states
//! - Event processing and queueing
//! - Custom actions and guards registration
//! - Context for storing runtime data
//!
//! # Example
//!
//! ```rust
//! use control_core::state_machine::{StateMachine, ActionRegistry, GuardRegistry};
//! use std::sync::Arc;
//!
//! let json = r#"
//! {
//!     "id": "traffic-light",
//!     "initial": "red",
//!     "states": {
//!         "red": {
//!             "entry": ["activateRedLight"],
//!             "on": { "TIMER": "green" }
//!         },
//!         "green": {
//!             "entry": ["activateGreenLight"],
//!             "on": { "TIMER": "yellow" }
//!         },
//!         "yellow": {
//!             "entry": ["activateYellowLight"],
//!             "on": { "TIMER": "red" }
//!         }
//!     }
//! }
//! "#;
//!
//! let mut machine = StateMachine::from_json(json).unwrap();
//!
//! // Register custom actions
//! // machine.actions_mut().register(Arc::new(MyAction));
//!
//! // Send events
//! machine.send("TIMER").unwrap();
//! assert_eq!(machine.current_state(), "green");
//! ```

mod actions;
mod machine;
mod types;

pub use actions::{
    Action, ActionRegistry, AlwaysTrueGuard, BoolGuard, Guard, GuardRegistry, LogAction,
    SetVariableAction,
};
pub use machine::StateMachine;
pub use types::{
    ContextValue, ExecutionState, MachineContext, StateConfig, StateMachineConfig, StateType,
    TransitionConfig,
};
