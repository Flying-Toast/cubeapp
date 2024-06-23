use crate::prelude::*;
use crate::stats::Penalty;
use std::cell::Cell;
use std::time::Duration;

glib::wrapper! {
    pub struct SolveStat(ObjectSubclass<SolveStatImp>);
}

impl SolveStat {
    pub fn new(tx: EventSender, time: Duration) -> Self {
        let this: Self = glib::Object::builder().build();
        let imp = this.imp();

        imp.time.set(time);
        imp.tx.set(Some(tx));

        let tx2 = this.get_tx();
        this.connect_notify(None, move |_, _| send_evt(tx2.clone(), Event::StatsChanged));

        this
    }

    fn get_tx(&self) -> EventSender {
        let tx = self.imp().tx.take().unwrap();
        self.imp().tx.set(Some(tx.clone()));
        tx
    }

    pub fn get_time(&self) -> Duration {
        let mut ret = self.imp().time.get();
        if self.penalty() == Penalty::Plus2 {
            ret += Duration::from_secs(2);
        }

        ret
    }
}

#[derive(Default, glib::Properties)]
#[properties(wrapper_type = SolveStat)]
pub struct SolveStatImp {
    time: Cell<Duration>,
    #[property(get, set, builder(Penalty::None))]
    penalty: Cell<Penalty>,
    tx: Cell<Option<EventSender>>,
}

#[glib::object_subclass]
impl ObjectSubclass for SolveStatImp {
    const NAME: &'static str = "PuzzleTimeSolveStat";
    type Type = SolveStat;
    type ParentType = glib::Object;
}

impl ObjectImpl for SolveStatImp {
    fn constructed(&self) {
        self.obj().connect_notify(Some("penalty"), |s, _| {
            s.notify("time-string");
        });
    }

    fn properties() -> &'static [glib::ParamSpec] {
        use std::sync::OnceLock;

        static PROPERTIES: OnceLock<Vec<glib::ParamSpec>> = OnceLock::new();
        PROPERTIES.get_or_init(|| {
            Self::derived_properties()
                .iter()
                .cloned()
                .chain([
                    glib::ParamSpecString::builder("time-string")
                        .read_only()
                        .build(),
                    glib::ParamSpecBoolean::builder("is-dnf")
                        .readwrite()
                        .build(),
                    glib::ParamSpecBoolean::builder("is-plus2")
                        .readwrite()
                        .build(),
                ])
                .collect()
        })
    }

    fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            "time-string" => crate::timer::render_time(&self.obj().get_time(), true).to_value(),
            "is-dnf" => (self.obj().penalty() == Penalty::Dnf).to_value(),
            "is-plus2" => (self.obj().penalty() == Penalty::Plus2).to_value(),
            _ => self.derived_property(id, pspec),
        }
    }

    fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
        match pspec.name() {
            "is-plus2" => {
                let value = value.get::<bool>().unwrap();
                match (value, self.penalty.get()) {
                    (true, Penalty::Dnf) => {
                        self.penalty.set(Penalty::Plus2);
                        self.obj().notify_penalty();
                        self.obj().notify("is-dnf");
                    }
                    (true, Penalty::None) => {
                        self.penalty.set(Penalty::Plus2);
                        self.obj().notify_penalty();
                    }
                    (false, Penalty::Plus2) => {
                        self.penalty.set(Penalty::None);
                        self.obj().notify_penalty();
                    }
                    _ => {}
                }
            }
            "is-dnf" => {
                let value = value.get::<bool>().unwrap();
                match (value, self.penalty.get()) {
                    (true, Penalty::Plus2) => {
                        self.penalty.set(Penalty::Dnf);
                        self.obj().notify_penalty();
                        self.obj().notify("is-plus2");
                    }
                    (true, Penalty::None) => {
                        self.penalty.set(Penalty::Dnf);
                        self.obj().notify_penalty();
                    }
                    (false, Penalty::Dnf) => {
                        self.penalty.set(Penalty::None);
                        self.obj().notify_penalty();
                    }
                    _ => {}
                }
            }
            _ => self.derived_set_property(id, value, pspec),
        }
    }
}
