use crate::prelude::*;
pub use crate::stat_object::SolveStat;
use std::time::Duration;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default, glib::Enum)]
#[enum_type(name = "PuzzleTimePenalty")]
pub enum Penalty {
    #[default]
    None,
    Dnf,
    Plus2,
}

#[derive(Debug)]
pub struct Stats {
    root: gtk::Box,
    store: gio::ListStore,
    backup: Option<(u32, SolveStat)>,
    ao5_label: gtk::Label,
    best_ao5: gtk::Label,
    session_average_label: gtk::Label,
}

impl Stats {
    pub fn new(tx: EventSender) -> Self {
        let builder = gtk::Builder::from_resource("/io/github/flying-toast/puzzle-time/stats.ui");
        let statuspage: adw::StatusPage = builder.object("statuspage").unwrap();
        let listview_factory: gtk::SignalListItemFactory =
            builder.object("listview_factory").unwrap();
        listview_factory.connect_setup(|_factory, list_item| {
            let list_item = list_item.downcast_ref::<gtk::ListItem>().unwrap();
            let ui = StatItemUi::new();
            list_item.set_child(Some(&ui.root));
            unsafe {
                list_item.set_data("PuzzleTimeUiStruct", ui);
            }
        });
        let tx2 = tx.clone();
        listview_factory.connect_bind(move |_factory, list_item| {
            let list_item = list_item.downcast_ref::<gtk::ListItem>().unwrap();
            let mut ui: StatItemUi = unsafe { list_item.steal_data("PuzzleTimeUiStruct") }.unwrap();
            let item = list_item.item().unwrap().downcast::<SolveStat>().unwrap();
            let my_index = list_item.position();

            let tx2 = tx2.clone();
            ui.click_handler = Some(ui.gestureclick.connect_released(move |_, _, _, _| {
                send_evt(tx2.clone(), Event::ShowStat(my_index));
            }));

            ui.index_label.set_label(&format!("{}.", my_index + 1));

            ui.bindings = vec![
                item.bind_property("is-dnf", &ui.dnf_button, "active")
                    .bidirectional()
                    .sync_create()
                    .build(),
                item.bind_property("is-plus2", &ui.plus2_button, "active")
                    .bidirectional()
                    .sync_create()
                    .build(),
                item.bind_property("penalty", &ui.dnf_button, "css-classes")
                    .transform_to(|_, penalty: Penalty| {
                        Some(if penalty == Penalty::Dnf {
                            ["error"].as_slice()
                        } else {
                            ["dim-label"].as_slice()
                        })
                    })
                    .sync_create()
                    .build(),
                item.bind_property("penalty", &ui.plus2_button, "css-classes")
                    .transform_to(|_, penalty: Penalty| {
                        Some(if penalty == Penalty::Plus2 {
                            ["warning"].as_slice()
                        } else {
                            ["dim-label"].as_slice()
                        })
                    })
                    .sync_create()
                    .build(),
                item.bind_property("penalty", &ui.time_label, "css-classes")
                    .transform_to(|_, penalty: Penalty| {
                        Some(match penalty {
                            Penalty::None => ["numeric"].as_slice(),
                            Penalty::Plus2 => ["warning", "numeric"].as_slice(),
                            Penalty::Dnf => ["strikethrough", "error", "numeric"].as_slice(),
                        })
                    })
                    .sync_create()
                    .build(),
                item.bind_property("time-string", &ui.time_label, "label")
                    .sync_create()
                    .build(),
            ];

            unsafe {
                list_item.set_data("PuzzleTimeUiStruct", ui);
            }
        });
        listview_factory.connect_unbind(move |_factory, list_item| {
            let list_item = list_item.downcast_ref::<gtk::ListItem>().unwrap();
            let mut ui: StatItemUi = unsafe { list_item.steal_data("PuzzleTimeUiStruct") }.unwrap();

            std::mem::take(&mut ui.bindings)
                .iter()
                .for_each(glib::Binding::unbind);
            ui.gestureclick.disconnect(ui.click_handler.take().unwrap());

            unsafe {
                list_item.set_data("PuzzleTimeUiStruct", ui);
            }
        });
        let store = gio::ListStore::new::<SolveStat>();
        let listview_model: gtk::NoSelection = gtk::NoSelection::new(Some(store.clone()));
        let listview: gtk::ListView = builder.object("listview").unwrap();
        listview_model
            .bind_property("n-items", &statuspage, "visible")
            .transform_to(|_, n: u32| Some(n == 0))
            .sync_create()
            .build();
        listview.set_model(Some(&listview_model));
        listview.set_factory(Some(&listview_factory));

        let tx2 = tx.clone();
        store.connect_notify(Some("n-items"), move |_, _| {
            send_evt(tx2.clone(), Event::StatsChanged);
        });

        Self {
            root: builder.object("root").unwrap(),
            store,
            backup: None,
            session_average_label: builder.object("session_average_label").unwrap(),
            ao5_label: builder.object("ao5_label").unwrap(),
            best_ao5: builder.object("best_ao5").unwrap(),
        }
    }

    pub fn widget(&self) -> &impl IsA<gtk::Widget> {
        &self.root
    }

    pub fn append_stat(&mut self, stat: &SolveStat) {
        self.backup = None;
        self.store.append(stat)
    }

    pub fn insert_stat(&mut self, idx: u32, stat: &SolveStat) {
        self.backup = None;
        self.store.insert(idx, stat)
    }

    pub fn get_stat(&self, index: u32) -> Option<SolveStat> {
        self.store.item(index).and_downcast::<SolveStat>()
    }

    pub fn remove(&self, index: u32) {
        self.store.remove(index);
    }

    fn num_nondnf(&self) -> u32 {
        self.fold_stats(0, |acc, s| {
            if s.penalty() == Penalty::Dnf {
                acc
            } else {
                acc + 1
            }
        })
    }

    pub fn set_backup(&mut self, backup: (u32, SolveStat)) {
        self.backup = Some(backup);
    }

    pub fn take_backup(&mut self) -> Option<(u32, SolveStat)> {
        self.backup.take()
    }

    /// Returns the number of stats that are in the store
    pub fn length(&self) -> u32 {
        self.store.n_items()
    }

    pub fn fold_stats<T, F: Fn(T, SolveStat) -> T>(&self, init: T, f: F) -> T {
        let mut acc = init;
        for i in 0..self.length() {
            acc = f(acc, self.get_stat(i).unwrap());
        }
        acc
    }

    pub fn update_stats(&self) {
        self.ao5_label.set_label("12345");
        self.best_ao5.set_label("45678");
        if self.num_nondnf() > 0 {
            self.session_average_label
                .set_label(&crate::timer::render_time(&self.average_time(), true));
        } else {
            self.session_average_label.set_label("DNF");
        }
    }

    pub fn average_time(&self) -> Duration {
        let sum = self.fold_stats(Duration::default(), |acc, stat| {
            acc + if stat.penalty() == Penalty::Dnf {
                Duration::default()
            } else {
                stat.get_time()
            }
        });

        sum / self.num_nondnf()
    }
}

#[derive(Debug)]
struct StatItemUi {
    root: gtk::Box,
    time_label: gtk::Label,
    index_label: gtk::Label,
    dnf_button: gtk::Button,
    plus2_button: gtk::Button,
    bindings: Vec<glib::Binding>,
    gestureclick: gtk::GestureClick,
    click_handler: Option<glib::SignalHandlerId>,
}

impl StatItemUi {
    fn new() -> Self {
        let builder =
            gtk::Builder::from_resource("/io/github/flying-toast/puzzle-time/stat-item.ui");

        Self {
            bindings: Vec::new(),
            click_handler: None,
            root: builder.object("root").unwrap(),
            time_label: builder.object("time_label").unwrap(),
            index_label: builder.object("index_label").unwrap(),
            dnf_button: builder.object("dnf_btn").unwrap(),
            plus2_button: builder.object("plus2_btn").unwrap(),
            gestureclick: builder.object("gestureclick").unwrap(),
        }
    }
}

/// `index`: index of the given `stat`
pub fn stat_info_dialog(tx: EventSender, stat: &SolveStat, index: u32) -> adw::Dialog {
    let builder =
        gtk::Builder::from_resource("/io/github/flying-toast/puzzle-time/stat-info-dialog.ui");
    let root = builder.object::<adw::Dialog>("root").unwrap();
    let delete_button: gtk::Button = builder.object("delete_button").unwrap();

    root.set_title(&format!("Result {}", index + 1));

    let root2 = root.clone();
    delete_button.connect_clicked(move |_| {
        root2.close();
        send_evt(tx.clone(), Event::DeleteStat(index));
    });

    root
}
