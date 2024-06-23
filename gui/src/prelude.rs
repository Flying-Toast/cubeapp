pub use crate::Event;
pub use adw::prelude::*;
pub use futures::prelude::*;
pub use gtk::subclass::prelude::*;
pub use gtk::{gdk, gio, glib};

pub type EventSender = futures::channel::mpsc::UnboundedSender<Event>;

pub fn send_evt(mut tx: EventSender, evt: Event) {
    glib::spawn_future(async move {
        tx.send(evt).await.unwrap();
    });
}
