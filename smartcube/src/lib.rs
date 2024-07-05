///! Utils for generically interacting with smartcubes
use btleplug::api::{Central as _, CentralEvent, Manager as _, Peripheral as _};
use btleplug::platform::{Adapter, Manager, Peripheral, PeripheralId};
use futures::stream::{Stream, StreamExt};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub trait Driver: std::fmt::Debug + Send + Sync {
    /// Name of this driver
    fn name(&self) -> &'static str;

    /// Returns whether or not this driver is compatible with the given peripheral.
    fn check_compat<'a>(
        &self,
        perip: &'a Peripheral,
    ) -> Pin<Box<dyn Future<Output = bool> + Send + 'a>>;

    /// Subscribe to events from the driver. The passed `Peripheral` is already connected.
    fn events(&self, perip: Peripheral) -> Pin<Box<dyn Stream<Item = SmartcubeEvent> + Send>>;
}

#[derive(Debug)]
pub enum SmartcubeEvent {
    /// New battery level in 0..=100
    Battery(u8),
    /// State change with timestamp
    StateChange(cubestruct::CubieCube, std::time::Instant),
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct DeviceId(PeripheralId);

#[derive(Debug, Clone)]
pub struct Device {
    perip: Peripheral,
    driver: &'static dyn Driver,
    local_name: String,
}

impl Device {
    pub fn id(&self) -> DeviceId {
        DeviceId(self.perip.id())
    }

    pub fn driver_name(&self) -> &'static str {
        self.driver.name()
    }

    pub fn local_name(&self) -> &str {
        &self.local_name.trim()
    }

    /// Connect to the device and start receiving events
    pub async fn connect(&self) -> impl Stream<Item = SmartcubeEvent> + Send + 'static {
        self.perip.connect().await.unwrap();

        self.driver.events(self.perip.clone())
    }

    pub async fn disconnect(&self) {
        self.perip.disconnect().await.unwrap();
    }

    async fn new(perip: Peripheral, driver: &'static dyn Driver) -> Self {
        let local_name = perip
            .properties()
            .await
            .unwrap()
            .unwrap()
            .local_name
            .unwrap();

        Self {
            perip,
            driver,
            local_name,
        }
    }
}

pub async fn init_bluetooth(drivers: &'static [&'static dyn Driver]) -> BluetoothManager {
    BluetoothManager::new(drivers).await
}

#[derive(Debug)]
pub enum ConnectionEvent {
    Connect(DeviceId),
    Disconnect(DeviceId),
    Discovery(Device),
}

#[derive(Debug, Clone)]
pub struct BluetoothManager {
    drivers: &'static [&'static dyn Driver],
    adapter: Arc<Adapter>,
}

impl BluetoothManager {
    pub fn events(
        &self,
    ) -> impl Future<Output = impl Stream<Item = ConnectionEvent> + Send + 'static> + 'static {
        let adapter = Arc::clone(&self.adapter);
        let drivers = self.drivers;

        async move {
            adapter
                .events()
                .await
                .unwrap()
                .filter_map(move |evt| filter_map_event(drivers, Arc::clone(&adapter), evt))
        }
    }

    pub async fn start_scan(&self) {
        self.adapter.start_scan(Default::default()).await.unwrap();
    }

    pub async fn stop_scan(&self) {
        self.adapter.stop_scan().await.unwrap();
    }

    async fn new(drivers: &'static [&'static dyn Driver]) -> Self {
        let adapter = Arc::new(
            Manager::new()
                .await
                .unwrap()
                .adapters()
                .await
                .unwrap()
                .into_iter()
                .nth(0)
                .expect("Can't get bluetooth adapter"),
        );

        Self { drivers, adapter }
    }
}

async fn filter_map_event(
    drivers: &'static [&'static dyn Driver],
    adapter: Arc<Adapter>,
    evt: CentralEvent,
) -> Option<ConnectionEvent> {
    match evt {
        CentralEvent::DeviceDiscovered(perip_id) => {
            let perip = adapter.peripheral(&perip_id).await.unwrap();

            make_device_if_supported(drivers, perip)
                .await
                .map(ConnectionEvent::Discovery)
        }
        CentralEvent::DeviceConnected(perip_id) => {
            Some(ConnectionEvent::Connect(DeviceId(perip_id)))
        }
        CentralEvent::DeviceDisconnected(perip_id) => {
            Some(ConnectionEvent::Disconnect(DeviceId(perip_id)))
        }
        _ => None,
    }
}

async fn make_device_if_supported(
    drivers: &'static [&'static dyn Driver],
    perip: Peripheral,
) -> Option<Device> {
    for driver in drivers {
        if driver.check_compat(&perip).await {
            return Some(Device::new(perip, *driver).await);
        }
    }

    None
}
