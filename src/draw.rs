use crate::app::{GameState, WordleCell};
use eframe::egui;
use eframe::egui::style::Margin;
use eframe::egui::{Color32, Response, RichText, Sense, Ui, Vec2};

pub(crate) fn draw_letters(ui: &mut Ui, cells: &[[WordleCell; 5]; 6]) {
    #[rustfmt::skip] // rustfmt wants to put none(), .margin() and .show() all on one line
    egui::Frame::none()
        .margin(Margin::symmetric(ui.available_width() / 2.0 - 128.0, 12.0))
        .show(ui, |ui| {
            egui::Grid::new("wordle_grid").spacing((4.0, 4.0)).show(ui, |ui| {
                for row in cells {
                    for cell in row {
                        // Draw a cell
                        egui::Frame::none()
                            .fill(cell.state.get_color(false))
                            .rounding(6.0)
                            .show(ui, |ui| {
                                add_letter_label(
                                    ui,
                                    (48.0, 48.0),
                                    &cell.letter.to_string(),
                                    36.0,
                                    false,
                                );
                            });
                    }
                    ui.end_row();
                }
            });
        });
}

pub(crate) fn draw_keyboard(
    ui: &mut Ui,
    keyboard: &([WordleCell; 10], [WordleCell; 9], [WordleCell; 7]),
    kbd_keydown: &mut String,
) {
    #[rustfmt::skip] // rustfmt wants to put none(), .margin() and .show() all on one line
    egui::Frame::none()
        .margin(Margin::symmetric(ui.available_width() / 2.0 - 198.0, 0.0))
        .show(ui, |ui| {
            // Add first row
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = (4.0, 0.0).into();
                add_keyboard_row(ui, &keyboard.0, kbd_keydown);
            });

            // Add second row
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = (4.0, 0.0).into();
                egui::Frame::none().margin(Margin::symmetric(20.0, 0.0)).show(
                    ui,
                    |ui| {
                        add_keyboard_row(ui, &keyboard.1, kbd_keydown);
                    },
                );
            });

            // Add third row
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = (4.0, 0.0).into();
                add_keyboard_button(ui, "ENT", true, Color32::GRAY, kbd_keydown);
                add_keyboard_row(ui, &keyboard.2, kbd_keydown);
                add_keyboard_button(ui, "DEL", true, Color32::GRAY, kbd_keydown);
            });
        });
}

fn add_keyboard_row(ui: &mut Ui, row: &[WordleCell], kbd_keydown: &mut String) {
    for key in row {
        add_keyboard_button(
            ui,
            &key.letter.to_string(),
            false,
            key.state.get_color(true),
            kbd_keydown,
        )
    }
}

fn add_keyboard_button(
    ui: &mut Ui,
    text: &str,
    big: bool,
    color: Color32,
    kbd_keydown: &mut String,
) {
    #[rustfmt::skip] // rustfmt wants to format 'if ui.add_sized' weird
    egui::Frame::none().fill(color).rounding(6.0).show(ui, |ui| {
        if add_letter_label(
            ui,
            (if big { 56.0 } else { 36.0 }, 48.0),
            text,
            18.0,
            true,
        ).clicked() && kbd_keydown.is_empty() {
            *kbd_keydown = text.to_string();
        }
    });
}

pub(crate) fn draw_game_end_message(ui: &mut Ui, game_state: &GameState, word: &str) {
    #[rustfmt::skip] // rustfmt wants to put none(), .margin() and .show() all on one line
    egui::Frame::none()
        .margin(Margin { left: 0.0, right: 0.0, top: 12.0, bottom: 24.0 })
        .show(ui, |ui| {
            ui.add_sized((400.0, 30.0), game_state.get_state_label(word));
        });
}

pub fn add_letter_label(
    ui: &mut Ui,
    size: impl Into<Vec2>,
    text: &str,
    text_size: f32,
    clickable: bool,
) -> Response {
    ui.add_sized(size, {
        let label =
            egui::Label::new(RichText::new(text).size(text_size).strong().color(Color32::WHITE));
        if clickable {
            label.sense(Sense::click())
        } else {
            label
        }
    })
}
