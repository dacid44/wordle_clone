use eframe::egui::{Color32, Key, Layout, RichText};
use eframe::{egui, epi};
use std::collections::HashMap;
use crate::{draw, utils};

const LETTERS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const KBD_ROW1: &str = "QWERTYUIOP";
const KBD_ROW2: &str = "ASDFGHJKL";
const KBD_ROW3: &str = "ZXCVBNM";

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
//#[cfg_attr(feature = "persistence", serde(default))]
#[derive(Debug, Clone)]
pub(crate) enum CellState {
    Empty,
    Gray,
    Yellow,
    Green,
}

impl CellState {
    pub(crate) fn get_color(&self, keyboard: bool) -> Color32 {
        match self {
            Self::Empty => {
                if keyboard {
                    Color32::GRAY
                } else {
                    Color32::BLACK
                }
            }
            Self::Gray => Color32::DARK_GRAY,
            Self::Yellow => Color32::from_rgb(181, 159, 59),
            Self::Green => Color32::from_rgb(83, 141, 78),
        }
    }
}

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
pub(crate) enum GameState {
    Playing,
    Success(usize),
    Failure,
}

impl GameState {
    pub(crate) fn get_state_label(&self, correct_word: &str) -> egui::Label {
        match self {
            Self::Playing => egui::Label::new(""),
            Self::Success(attempts) => egui::Label::new(
                RichText::new(format!("Success in {} tries!", attempts))
                    .size(24.0)
                    .strong()
                    .color(Color32::DARK_GREEN),
            ),
            Self::Failure => egui::Label::new(
                RichText::new(format!("The correct word was \"{}\".", correct_word))
                    .size(24.0)
                    .strong()
                    .color(Color32::RED),
            ),
        }
    }
}

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))]
#[derive(Default)]
struct DebugMenu {
    open: bool,
    #[cfg_attr(feature = "persistence", serde(skip))]
    new_word: String,
    #[cfg_attr(feature = "persistence", serde(skip))]
    focus: bool,
}

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))]
#[derive(Debug)]
pub(crate) struct WordleCell {
    pub state: CellState,
    pub letter: char,
}

impl WordleCell {
    pub(crate) fn keyboard(c: char) -> Self {
        Self { state: CellState::Empty, letter: c }
    }
}

impl Default for WordleCell {
    fn default() -> Self {
        Self { state: CellState::Empty, letter: ' ' }
    }
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct WordleApp {
    #[cfg_attr(feature = "persistence", serde(skip))]
    pub(crate) args: utils::Args,
    #[cfg_attr(feature = "persistence", serde(skip))]
    pub(crate) word: String,
    #[cfg_attr(feature = "persistence", serde(skip))]
    pub(crate) cells: [[WordleCell; 5]; 6],
    #[cfg_attr(feature = "persistence", serde(skip))]
    pub(crate) next_cell: (usize, usize),
    #[cfg_attr(feature = "persistence", serde(skip))]
    pub(crate) keyboard: ([WordleCell; 10], [WordleCell; 9], [WordleCell; 7]),
    #[cfg_attr(feature = "persistence", serde(skip))]
    pub(crate) keyboard_state: HashMap<char, CellState>,
    #[cfg_attr(feature = "persistence", serde(skip))]
    pub(crate) kbd_keydown: String,
    #[cfg_attr(feature = "persistence", serde(skip))]
    pub(crate) game_state: GameState,
    debug_menu: DebugMenu,
}

impl Default for WordleApp {
    fn default() -> Self {
        Self {
            args: Default::default(),
            word: utils::get_random_word(),
            cells: Default::default(),
            next_cell: (0, 0),
            #[rustfmt::skip] // rustfmt wants to split these chains into loads of tiny lines
            keyboard: (
                KBD_ROW1.chars().map(WordleCell::keyboard)
                    .collect::<Vec<WordleCell>>().try_into().unwrap(),
                KBD_ROW2.chars().map(WordleCell::keyboard)
                    .collect::<Vec<WordleCell>>().try_into().unwrap(),
                KBD_ROW3.chars().map(WordleCell::keyboard)
                    .collect::<Vec<WordleCell>>().try_into().unwrap(),
            ),
            keyboard_state: LETTERS.chars().zip(std::iter::repeat(CellState::Empty)).collect(),
            kbd_keydown: String::default(),
            game_state: GameState::Playing,
            debug_menu: DebugMenu::default(),
        }
    }
}

impl epi::App for WordleApp {
    fn name(&self) -> &str {
        "Wordle Clone"
    }

    /// Called once before the first frame.
    fn setup(
        &mut self,
        _ctx: &egui::Context,
        _frame: &epi::Frame,
        _storage: Option<&dyn epi::Storage>,
    ) {
        let args = self.args.clone();

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        #[cfg(feature = "persistence")]
        if let Some(storage) = _storage {
            *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
        }

        self.args = args;
        if let Some(word) = &self.args.word {
            self.word = word.clone();
        }
    }

    /// Called by the frame work to save state before shutdown.
    /// Note that you must enable the `persistence` feature for this to work.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, frame: &epi::Frame) {
        if self.debug_menu.open {
            egui::Window::new("Debug Menu").show(ctx, |ui| {
                ui.label(format!("Current word: {}", self.word));
                ui.label(format!(
                    "Current word (base64): {}",
                    utils::encode(self.word.clone()),
                ));
                ui.label(format!(
                    "Current word (base64 decoded): {}",
                    utils::decode(utils::encode(self.word.clone())).unwrap(),
                ));

                let response = ui.text_edit_singleline(&mut self.debug_menu.new_word);
                self.debug_menu.focus = response.has_focus();
                if response.lost_focus()
                    && ui.input().key_pressed(Key::Enter)
                    && crate::WORD_LIST.contains(&&*self.debug_menu.new_word.to_uppercase())
                {
                    self.word = self.debug_menu.new_word.to_uppercase();
                }

                if ui.button("Reset").clicked() {
                    self.reset();
                }
                if ui.button("Reset with random word").clicked() {
                    self.reset_random_word();
                }
            });
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Menu", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                    ui.checkbox(&mut self.debug_menu.open, "Show debug menu");
                });
                ui.with_layout(Layout::right_to_left(), |ui| {
                    egui::global_dark_light_mode_switch(ui);
                })
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            if matches!(self.game_state, GameState::Playing)
                && (!self.debug_menu.open || !self.debug_menu.focus)
            {
                self.handle_keys(ui)
            }
            // If there was a key found, clear it
            if !self.kbd_keydown.is_empty() {
                self.kbd_keydown = String::new();
            }

            ui.vertical_centered(|ui| {
                ui.set_width(ui.available_width());
                draw::draw_letters(ui, &self.cells);
                draw::draw_game_end_message(ui, &self.game_state, &self.word);
                draw::draw_keyboard(ui, &self.keyboard, &mut self.kbd_keydown);
            });
            egui::warn_if_debug_build(ui);
        });
    }
}
