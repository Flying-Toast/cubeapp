use crate::prelude::*;
use futures::stream::StreamExt;
use smartcube::{BluetoothManager, DeviceId, SmartcubeEvent};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::OnceLock;
use tokio::runtime::Runtime;

#[derive(Debug)]
pub struct Bluetooth {
    dialog: adw::Dialog,
    tx: EventSender,
    device_listbox: gtk::ListBox,
    manager: Option<BluetoothManager>,
    known_devices: HashMap<DeviceId, DeviceInfo>,
    did_init: bool,
    toaster: adw::ToastOverlay,
}

#[derive(Debug)]
struct DeviceInfo {
    spinner: gtk::Spinner,
    switch: gtk::Switch,
    device: smartcube::Device,
}

fn tokio() -> &'static Runtime {
    static RUNTIME: OnceLock<Runtime> = OnceLock::new();
    RUNTIME.get_or_init(|| Runtime::new().unwrap())
}

impl Bluetooth {
    pub fn new(tx: EventSender, toaster: adw::ToastOverlay) -> Self {
        let builder =
            gtk::Builder::from_resource("/io/github/flying-toast/puzzle-time/bluetooth-dialog.ui");
        let dialog: adw::Dialog = builder.object("root").unwrap();
        let tx2 = tx.clone();
        dialog.connect_closed(move |_| {
            send_evt(tx2.clone(), Event::StopBluetoothScan);
        });
        Self {
            tx,
            toaster,
            dialog,
            device_listbox: builder.object("device_list").unwrap(),
            known_devices: HashMap::new(),
            did_init: false,
            manager: None,
        }
    }

    pub fn dialog(&self) -> &adw::Dialog {
        &self.dialog
    }

    pub fn maybe_init(&mut self) {
        if self.did_init {
            return;
        }
        self.did_init = true;
        let mut tx = self.tx.clone();
        tokio().spawn(async move {
            let manager = smartcube::init_bluetooth(&[&qiyi_smartcube::Driver]).await;
            tx.send(Event::BluetoothInitialized(manager)).await.unwrap();
        });
    }

    pub fn manager_ready(&mut self, manager: BluetoothManager) {
        let mut tx = self.tx.clone();
        let manager2 = manager.clone();
        tokio().spawn(async move {
            let mut events = std::pin::pin!(manager2.events().await);
            manager2.start_scan().await;
            while let Some(evt) = events.next().await {
                match evt {
                    smartcube::ConnectionEvent::Discovery(dev) => {
                        tx.send(Event::BluetoothDeviceDiscoverd(dev)).await.unwrap()
                    }
                    smartcube::ConnectionEvent::Connect(id) => {
                        tx.send(Event::BluetoothDeviceConnected(id)).await.unwrap()
                    }
                    smartcube::ConnectionEvent::Disconnect(id) => tx
                        .send(Event::BluetoothDeviceDisconnected(id))
                        .await
                        .unwrap(),
                }
            }
            panic!("Manager event stream ended");
        });
        assert!(self.manager.is_none());
        self.manager = Some(manager);
    }

    pub fn add_discovered_device(&mut self, dev: smartcube::Device) {
        let row = adw::ActionRow::builder()
            .activatable(true)
            .title(dev.local_name())
            .subtitle(dev.driver_name())
            .build();
        let switch = gtk::Switch::new();
        switch.set_valign(gtk::Align::Center);
        let task_handle = Arc::new(RefCell::new(None));
        let spinner = gtk::Spinner::new();
        self.known_devices.insert(
            dev.id(),
            DeviceInfo {
                spinner: spinner.clone(),
                device: dev.clone(),
                switch: switch.clone(),
            },
        );
        let app_tx = self.tx.clone();
        let spinner2 = spinner.clone();
        switch.connect_state_set(move |me, state| {
            me.set_sensitive(false);
            spinner2.set_spinning(true);
            let dev = dev.clone();
            if state {
                let mut app_tx = app_tx.clone();
                assert!(
                    task_handle.borrow().is_none(),
                    "Tried to connect to device but it already has a running task"
                );
                *task_handle.borrow_mut() = Some(tokio().spawn(async move {
                    let mut events = dev.connect().await;
                    while let Some(evt) = events.next().await {
                        app_tx.send(Event::Smartcube(evt)).await.unwrap();
                    }
                    panic!("Device event stream ended");
                }));
            } else {
                if let Some(handle) = task_handle.borrow_mut().take() {
                    handle.abort();
                }
                tokio().spawn(async move {
                    dev.disconnect().await;
                });
            }

            glib::Propagation::Proceed
        });
        row.add_suffix(&spinner);
        row.add_suffix(&switch);
        self.device_listbox.append(&row);
    }

    pub fn device_connected(&self, id: DeviceId) {
        let info = self.known_devices.get(&id).unwrap();
        info.switch.set_active(true);
        info.switch.set_sensitive(true);
        info.spinner.set_spinning(false);
        let toast = adw::Toast::new(&format!("Connected to {}", info.device.local_name()));
        self.toaster.add_toast(toast);
    }

    pub fn device_disconnected(&self, id: DeviceId) {
        let info = self.known_devices.get(&id).unwrap();
        info.switch.set_active(false);
        info.switch.set_sensitive(true);
        info.spinner.set_spinning(false);
        let toast = adw::Toast::new(&format!("{} Disconnected", info.device.local_name()));
        self.toaster.add_toast(toast);
    }

    pub fn handle_smartcube_event(&self, evt: SmartcubeEvent) {
        dbg!(evt);
    }

    pub fn start_scan(&self) {
        if let Some(manager) = self.manager.clone() {
            tokio().spawn(async move {
                manager.start_scan().await;
            });
        }
    }

    pub fn stop_scan(&self) {
        let manager = self.manager.clone().unwrap();
        tokio().spawn(async move {
            manager.stop_scan().await;
        });
    }
}
