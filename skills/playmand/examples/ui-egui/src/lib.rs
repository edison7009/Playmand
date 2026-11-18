//! Headless egui recipes for debug panels and tools. egui is immediate mode:
//! the UI is rebuilt every frame and each widget returns what happened to it.
//! These tests feed real pointer and text events into one context, frame by
//! frame, and check the state the UI is bound to.
//!
//! egui 0.36 is the version bevy_egui 0.43 uses with Bevy 0.20.

#[cfg(test)]
mod tests {
    use egui::{Context, Event, Modifiers, PointerButton, Pos2, RawInput, Rect, Ui, vec2};

    /// One egui context driven by hand at 60 frames a second.
    struct Harness {
        ctx: Context,
        time: f64,
    }

    impl Harness {
        fn new() -> Self {
            Self {
                ctx: Context::default(),
                time: 0.0,
            }
        }

        fn frame(&mut self, events: Vec<Event>, ui: impl FnMut(&mut Ui)) {
            let input = RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(400.0, 300.0))),
                time: Some(self.time),
                events,
                ..Default::default()
            };
            self.time += 1.0 / 60.0;
            // Without a renderer nobody uploads the font textures, and an
            // unapplied texture delta panics when dropped.
            self.ctx.run_ui(input, ui).drop_without_applying_deltas();
        }
    }

    fn button(pos: Pos2, pressed: bool) -> Event {
        Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        }
    }

    /// Moves to `at`, presses and releases over three frames, returning on
    /// which of those frames `clicked` was true.
    fn click(h: &mut Harness, at: Pos2, mut ui: impl FnMut(&mut Ui) -> bool) -> Vec<bool> {
        let mut seen = Vec::new();
        for events in [
            vec![Event::PointerMoved(at)],
            vec![button(at, true)],
            vec![button(at, false)],
            vec![],
        ] {
            let mut clicked = false;
            h.frame(events, |u| clicked = ui(u));
            seen.push(clicked);
        }
        seen
    }

    #[test]
    fn a_button_reports_one_click_on_the_frame_it_is_released() {
        let mut h = Harness::new();
        let mut rect = Rect::NOTHING;
        h.frame(vec![], |ui| rect = ui.button("Spawn enemy").rect);
        let seen = click(&mut h, rect.center(), |ui| {
            ui.button("Spawn enemy").clicked()
        });
        assert_eq!(seen, [false, false, true, false]);
    }

    #[test]
    fn a_checkbox_flips_the_setting_it_is_bound_to() {
        let mut h = Harness::new();
        let mut show_colliders = false;
        let mut rect = Rect::NOTHING;
        h.frame(vec![], |ui| {
            rect = ui.checkbox(&mut show_colliders, "Show colliders").rect
        });
        click(&mut h, rect.center(), |ui| {
            ui.checkbox(&mut show_colliders, "Show colliders").changed()
        });
        assert!(show_colliders);
    }

    #[test]
    fn a_focused_text_field_receives_typed_text() {
        let mut h = Harness::new();
        let mut command = String::from("spawn");
        let mut rect = Rect::NOTHING;
        h.frame(vec![], |ui| {
            rect = ui.text_edit_singleline(&mut command).rect
        });
        click(&mut h, rect.center(), |ui| {
            ui.text_edit_singleline(&mut command).has_focus()
        });
        h.frame(vec![Event::Text(" boss".into())], |ui| {
            ui.text_edit_singleline(&mut command);
        });
        assert_eq!(command, "spawn boss");
    }
}
