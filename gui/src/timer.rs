use crate::prelude::*;
use std::fmt::Write;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct Timer {
    tx: EventSender,
    start_time: Option<Instant>,
    update_timeout: Option<glib::SourceId>,
    main_box: gtk::Box,
    redlight: adw::Bin,
    greenlight: adw::Bin,
    time_label: gtk::Label,
    scramble_label: gtk::Label,
    current_scramble: Vec<cubestruct::Move>,
}

impl Timer {
    pub fn new(tx: EventSender) -> Self {
        let builder = gtk::Builder::from_resource("/io/github/flying_toast/PuzzleTime/timer.ui");
        let mut this = Self {
            tx,
            main_box: builder.object("main_box").unwrap(),
            redlight: builder.object("redlight").unwrap(),
            greenlight: builder.object("greenlight").unwrap(),
            time_label: builder.object("time_label").unwrap(),
            scramble_label: builder.object("scramble").unwrap(),
            start_time: None,
            update_timeout: None,
            current_scramble: Vec::new(),
        };
        this.gen_new_scramble();
        this
    }

    pub fn widget(&self) -> &impl IsA<gtk::Widget> {
        &self.main_box
    }

    pub fn red_light_on(&self) {
        self.redlight
            .set_css_classes(["timer-light", "timer-light-red"].as_slice());
    }

    pub fn both_lights_on(&self) {
        self.red_light_on();
        self.greenlight
            .set_css_classes(["timer-light", "timer-light-green"].as_slice());
    }

    pub fn lights_off(&self) {
        self.redlight
            .set_css_classes(&["timer-light", "timer-light-off"].as_slice());
        self.greenlight
            .set_css_classes(&["timer-light", "timer-light-off"].as_slice());
    }

    pub fn start(&mut self) {
        assert!(!self.running(), "Timer already running");
        assert!(self.update_timeout.is_none());
        let tx = self.tx.clone();
        self.start_time = Some(Instant::now());
        self.update_timeout = Some(glib::timeout_add(Duration::from_millis(100), move || {
            send_evt(tx.clone(), Event::UpdateDisplayTime);
            glib::ControlFlow::Continue
        }));
    }

    pub fn stop(&mut self) -> Duration {
        self.update_timeout.take().unwrap().remove();
        let elapsed = self
            .start_time
            .take()
            .expect("Timer isn't running")
            .elapsed();
        self.set_displayed_time(&elapsed, true);

        elapsed
    }

    pub fn running(&self) -> bool {
        self.start_time.is_some()
    }

    pub fn update_displayed_time(&self) {
        if let Some(start_time) = &self.start_time {
            self.set_displayed_time(&start_time.elapsed(), false);
        }
    }

    fn set_displayed_time(&self, dur: &Duration, show_hunds: bool) {
        self.time_label.set_label(&render_time(dur, show_hunds));
    }

    pub fn current_scramble(&self) -> &[cubestruct::Move] {
        &self.current_scramble
    }

    /// Get the current scramble, replacing it with a newly generated random one
    pub fn take_scramble(&mut self) -> Vec<cubestruct::Move> {
        let ret = self.current_scramble.clone();
        self.gen_new_scramble();
        ret
    }

    fn gen_new_scramble(&mut self) {
        // TODO: actually generate scrambles lol
        if self.current_scramble.is_empty() {
            use cubestruct::Move::*;
            self.current_scramble = vec![
                D, F2, D2, Ui, F2, R2, F2, Ri, D, L, Bi, Li, B, F2, Di, Fi, L, Bi, Ui,
            ];
        }
        self.current_scramble.reverse();
        self.scramble_label
            .set_label(&render_moveseq(&self.current_scramble, true));
    }
}

pub fn render_time(dur: &Duration, show_hunds: bool) -> String {
    let mut rem = dur.as_millis() / 10;
    let hunds = rem % 100;
    rem /= 100;
    let secs = rem % 60;
    rem /= 60;
    let mins = rem;

    if mins == 0 {
        if show_hunds {
            format!("{secs}.{hunds:02}")
        } else {
            format!("{secs}.{}", hunds / 10)
        }
    } else {
        if show_hunds {
            format!("{mins}:{secs:02}.{hunds:02}")
        } else {
            format!("{mins}:{secs:02}.{}", hunds / 10)
        }
    }
}

pub fn render_moveseq(moves: &[cubestruct::Move], double_space: bool) -> String {
    let mut s = format!("{}", moves[0]);
    for moov in moves.iter().skip(1) {
        if double_space {
            write!(s, "  {moov}").unwrap();
        } else {
            write!(s, " {moov}").unwrap();
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn time_rendering() {
        let d_0m12s83 = Duration::from_secs(12) + Duration::from_millis(830);
        assert_eq!(render_time(&d_0m12s83, true), "12.83");

        let d_0m0s00 = Duration::default();
        assert_eq!(render_time(&d_0m0s00, true), "0.00");
        assert_eq!(render_time(&d_0m0s00, false), "0.0");

        let d_10m0s00 = Duration::from_secs(60 * 10);
        assert_eq!(render_time(&d_10m0s00, true), "10:00.00");
        assert_eq!(render_time(&d_10m0s00, false), "10:00.0");

        let d_0m1s09 = Duration::from_millis(1090);
        assert_eq!(render_time(&d_0m1s09, true), "1.09");
        assert_eq!(render_time(&d_0m1s09, false), "1.0");

        let d_0m4s30 = Duration::from_millis(4300);
        assert_eq!(render_time(&d_0m4s30, true), "4.30");
        assert_eq!(render_time(&d_0m4s30, false), "4.3");
    }
}
