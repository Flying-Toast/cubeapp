mod prelude;
mod stat_object;
mod stats;
mod timer;

use crate::prelude::*;
use futures::{channel::mpsc, stream::StreamExt};
use stats::SolveStat;
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum Event {
    SpacebarDown,
    SpacebarUp,
    /// A key other than spacebar was pressed
    NonSpacebarKeyDown,
    /// Update timer's displayed time
    UpdateDisplayTime,
    /// Send when it's time to turn on the timer's green light
    GreenLightReady,
    Quit,
    /// Show the stat at the given index
    ShowStat(u32),
    DeleteStat(u32),
    RestoreDeletedStat,
    StatsChanged,
}

#[derive(Debug)]
struct CubeApp {
    application: adw::Application,
    window: adw::ApplicationWindow,
    toasts: adw::ToastOverlay,
    timer: timer::Timer,
    stats: stats::Stats,
    spacebar_being_held: bool,
    greenlight_timeout: Option<glib::SourceId>,
    timer_ready: bool,
    tx: EventSender,
}

impl CubeApp {
    fn new(app: adw::Application, tx: EventSender) -> Self {
        let builder =
            gtk::Builder::from_resource("/io/github/flying-toast/puzzle-time/main-window.ui");
        let window: adw::ApplicationWindow = builder.object("window").unwrap();
        window.set_application(Some(&app));

        let quit_act = gio::SimpleAction::new("quit", None);
        let tx2 = tx.clone();
        quit_act.connect_activate(move |_, _| send_evt(tx2.clone(), Event::Quit));
        app.set_accels_for_action("app.quit", &["<Primary>Q"]);
        app.add_action(&quit_act);
        let remove_undo = gio::SimpleAction::new("undo-remove-stat", None);
        let tx2 = tx.clone();
        remove_undo.connect_activate(move |_, _| send_evt(tx2.clone(), Event::RestoreDeletedStat));
        app.add_action(&remove_undo);

        let timer = timer::Timer::new(tx.clone());

        let key_controller = gtk::EventControllerKey::new();
        key_controller.set_propagation_phase(gtk::PropagationPhase::Capture);
        let tx2 = tx.clone();
        key_controller.connect_key_pressed(move |_, key, _, _| {
            if key == gdk::Key::space {
                send_evt(tx2.clone(), Event::SpacebarDown);
                glib::Propagation::Stop
            } else {
                send_evt(tx2.clone(), Event::NonSpacebarKeyDown);
                glib::Propagation::Proceed
            }
        });
        let tx2 = tx.clone();
        key_controller.connect_key_released(move |_, key, _, _| {
            if key == gdk::Key::space {
                send_evt(tx2.clone(), Event::SpacebarUp);
            }
        });
        window.add_controller(key_controller);

        let stats = stats::Stats::new(tx.clone());
        let stats_pane: adw::OverlaySplitView = builder.object("stats_pane").unwrap();
        stats_pane.set_sidebar(Some(stats.widget()));
        stats_pane.set_content(Some(timer.widget()));

        window.present();

        Self {
            application: app,
            tx,
            timer,
            stats,
            spacebar_being_held: false,
            window,
            timer_ready: false,
            toasts: builder.object("toasts").unwrap(),
            greenlight_timeout: None,
        }
    }

    fn stop_timer(&mut self) {
        self.timer.lights_off();
        let elapsed_time = self.timer.stop();
        let stat = SolveStat::new(self.tx.clone(), elapsed_time);
        self.stats.append_stat(&stat);
    }
}

const TIMER_IDLE_HOLD_PERIOD: Duration = Duration::from_millis(500);

fn main() {
    gtk::init().unwrap();
    adw::init().unwrap();

    let application = adw::Application::builder()
        .application_id("io.github.flying-toast.puzzle-time")
        .build();

    application.connect_startup(|_| {
        gio::resources_register_include!("puzzle-time.gresource").unwrap();
        gtk::IconTheme::for_display(&gdk::Display::default().unwrap())
            .add_resource_path("/io/github/flying-toast/puzzle-time/icons");

        // load CSS
        let provider = gtk::CssProvider::new();
        provider.load_from_resource("/io/github/flying-toast/puzzle-time/main.css");
        gtk::style_context_add_provider_for_display(
            &gdk::Display::default().unwrap(),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    });

    let application2 = application.clone();

    application2.clone().connect_activate(move |_| {
        let (tx, mut rx) = mpsc::unbounded();
        let tx2 = tx.clone();
        let mut app = CubeApp::new(application.clone(), tx);

        glib::spawn_future_local(async move {
            while let Some(evt) = rx.next().await {
                match evt {
                    Event::Quit => {
                        app.application.quit();
                    }
                    Event::SpacebarDown => {
                        if app.spacebar_being_held {
                            continue;
                        }
                        app.spacebar_being_held = true;
                        app.timer_ready = false;

                        if app.timer.running() {
                            app.stop_timer();
                        } else {
                            app.timer.red_light_on();
                            let tx2 = tx2.clone();
                            app.greenlight_timeout =
                                Some(glib::timeout_add(TIMER_IDLE_HOLD_PERIOD, move || {
                                    send_evt(tx2.clone(), Event::GreenLightReady);
                                    glib::ControlFlow::Break
                                }));
                        }
                    }
                    Event::SpacebarUp => {
                        if !app.spacebar_being_held {
                            continue;
                        }
                        app.spacebar_being_held = false;

                        if !app.timer_ready {
                            if let Some(srcid) = app.greenlight_timeout.take() {
                                srcid.remove();
                            }
                            app.timer.lights_off();
                            continue;
                        }

                        app.timer_ready = false;
                        app.timer.start();
                    }
                    Event::NonSpacebarKeyDown => {
                        if app.timer.running() {
                            app.stop_timer();
                        }
                    }
                    Event::UpdateDisplayTime => {
                        app.timer.update_displayed_time();
                    }
                    Event::GreenLightReady => {
                        app.greenlight_timeout = None;
                        app.timer_ready = true;
                        app.timer.both_lights_on();
                    }
                    Event::ShowStat(idx) => {
                        stats::stat_info_dialog(
                            tx2.clone(),
                            &app.stats.get_stat(idx).unwrap(),
                            idx,
                        )
                        .present(&app.window);
                    }
                    Event::DeleteStat(idx) => {
                        let backup_stat = app.stats.get_stat(idx).unwrap();
                        app.stats.set_backup((idx, backup_stat));
                        app.stats.remove(idx);

                        let toast = adw::Toast::new(&format!("Result {} Deleted", idx + 1));
                        toast.set_button_label(Some("Undo"));
                        toast.set_action_name(Some("app.undo-remove-stat"));
                        app.toasts.add_toast(toast);
                    }
                    Event::RestoreDeletedStat => {
                        if let Some((idx, stat)) = app.stats.take_backup() {
                            app.stats.insert_stat(idx, &stat);
                        } else {
                            app.toasts
                                .add_toast(adw::Toast::new(&"Failed to Undo Deletion"));
                        }
                    }
                    Event::StatsChanged => {
                        app.stats.update_stats();
                    }
                }
            }
        });
    });
    application2.run();
}
