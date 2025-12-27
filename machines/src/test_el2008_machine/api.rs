use super::{MachineState, TestEL2008Machine, TestEL2008Mode};
use crate::{MachineApi, MachineMessage};
use control_core::socketio::{
    event::{Event, GenericEvent},
    namespace::{
        CacheFn, CacheableEvents, Namespace, NamespaceCacheingLogic, cache_first_and_last_event,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

#[derive(Serialize, Debug, Clone)]
pub struct StateEvent {
    pub led_on: [bool; 8],
    pub mode: TestEL2008Mode,
    pub machine_state: MachineState,
    pub automatic_delay_ms: u64,
}

impl StateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("StateEvent", self.clone())
    }
}

pub enum TestEL2008MachineEvents {
    State(Event<StateEvent>),
}

#[derive(Deserialize)]
#[serde(tag = "action", content = "value")]
pub enum Mutation {
    SetLed { index: usize, on: bool },
    SetAllLeds { on: bool },
    SetMode { mode: TestEL2008Mode },
    Start,
    Stop,
    Reset,
    SetAutomaticDelay { delay_ms: u64 },
}

#[derive(Debug, Clone)]
pub struct TestEL2008MachineNamespace {
    pub namespace: Option<Namespace>,
}

impl NamespaceCacheingLogic<TestEL2008MachineEvents> for TestEL2008MachineNamespace {
    fn emit(&mut self, events: TestEL2008MachineEvents) {
        let event = Arc::new(events.event_value());
        let buffer_fn = events.event_cache_fn();
        if let Some(ns) = &mut self.namespace {
            ns.emit(event, &buffer_fn);
        }
    }
}

impl CacheableEvents<TestEL2008MachineEvents> for TestEL2008MachineEvents {
    fn event_value(&self) -> GenericEvent {
        match self {
            TestEL2008MachineEvents::State(event) => event.clone().into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        cache_first_and_last_event()
    }
}

impl MachineApi for TestEL2008Machine {
    fn api_get_sender(&self) -> smol::channel::Sender<MachineMessage> {
        self.api_sender.clone()
    }

    fn api_mutate(&mut self, request_body: Value) -> Result<(), anyhow::Error> {
        let mutation: Mutation = serde_json::from_value(request_body)?;
        match mutation {
            Mutation::SetLed { index, on } => {
                self.set_led(index, on);
            }
            Mutation::SetAllLeds { on } => {
                self.set_all_leds(on);
            }
            Mutation::SetMode { mode } => {
                self.set_mode(mode);
            }
            Mutation::Start => {
                self.start();
            }
            Mutation::Stop => {
                self.stop();
            }
            Mutation::Reset => {
                self.reset();
            }
            Mutation::SetAutomaticDelay { delay_ms } => {
                self.set_automatic_delay(delay_ms);
            }
        }

        self.apply_outputs();

        Ok(())
    }

    fn api_event_namespace(&mut self) -> Option<Namespace> {
        self.namespace.namespace.clone()
    }
}
