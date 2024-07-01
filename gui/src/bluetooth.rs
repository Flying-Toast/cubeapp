use crate::prelude::*;
use futures::stream::StreamExt;
use smartcube::{BluetoothManager, Device, DeviceId};
use std::collections::HashMap;
use std::sync::OnceLock;
use tokio::runtime::Runtime;

#[derive(Debug)]
pub struct Bluetooth {
    dialog: adw::Dialog,
    tx: EventSender,
    device_listbox: gtk::ListBox,
    known_devices: HashMap<DeviceId, Device>,
    did_init: bool,
}

fn tokio() -> &'static Runtime {
    static RUNTIME: OnceLock<Runtime> = OnceLock::new();
    RUNTIME.get_or_init(|| Runtime::new().unwrap())
}

impl Bluetooth {
    pub fn new(tx: EventSender) -> Self {
        let builder =
            gtk::Builder::from_resource("/io/github/flying-toast/puzzle-time/bluetooth-dialog.ui");

        Self {
            tx,
            dialog: builder.object("root").unwrap(),
            device_listbox: builder.object("device_list").unwrap(),
            known_devices: HashMap::new(),
            did_init: false,
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
        tokio().spawn(async move {
            let mut events = std::pin::pin!(manager.events().await);
            manager.start_scan().await;
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
        });
    }

    pub fn add_discovered_device(&mut self, dev: smartcube::Device) {
        let row = adw::ActionRow::builder()
            .activatable(true)
            .title(dev.local_name())
            .subtitle(dev.driver_name())
            .build();
        let switch = gtk::Switch::new();
        switch.set_valign(gtk::Align::Center);
        let tx = self.tx.clone();
        switch.connect_state_set(move |me, state| {
            //me.set_sensitive(false);
            if state {
                println!("connecting...");
            } else {
                println!("disconnecting...");
            }

            glib::Propagation::Proceed
        });
        row.add_suffix(&switch);
        self.device_listbox.append(&row);
        self.known_devices.insert(dev.id(), dev);
    }

    pub fn connected(&mut self, id: DeviceId) {
        println!("DEVICE CONNECTED!");
    }

    pub fn disconnected(&mut self, id: DeviceId) {
        println!("By bye device");
    }
}
