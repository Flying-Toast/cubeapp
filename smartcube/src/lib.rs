#![allow(async_fn_in_trait)]
///! Traits for generically interacting with smartcubes
use anyhow::Result;
use btleplug::platform::Peripheral;
use cubestruct::CubeState;
use futures::stream::Stream;
use std::pin::Pin;

pub trait Smartcube {
    /// Is the given `Peripheral` an instance of this kind of smartcube?
    async fn matches_peripheral(perip: &Peripheral) -> Result<bool>;

    /// Start the protocol with this smartcube, get events
    async fn connect(perip: Peripheral) -> Result<Pin<Box<dyn Stream<Item = SmartcubeEvent>>>>;
}

pub enum SmartcubeEvent {
    /// New battery level (0..=100)
    Battery(u8),
    /// A move happened on the smartcube
    StateChange(CubeState, std::time::Instant),
}
