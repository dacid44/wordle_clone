use std::collections::HashMap;
use eframe::{egui, epi};
use eframe::egui::{Color32, Key, Layout, RichText, Sense, Ui};
use eframe::egui::style::Margin;
use rand::seq::SliceRandom;

const LETTERS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const KBD_ROW1: &str = "QWERTYUIOP";
const KBD_ROW2: &str = "ASDFGHJKL";
const KBD_ROW3: &str = "ZXCVBNM";

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
//#[cfg_attr(feature = "persistence", serde(default))]
#[derive(Debug, Clone)]
enum CellState {
    Empty,
    Gray,
    Yellow,
    Green,
}

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
enum GameState {
    Playing,
    Success(usize),
    Failure,
}

impl GameState {
    fn get_state_label(&self, correct_word: &String) -> egui::Label {
        match self {
            Self::Playing => egui::Label::new(""),
            Self::Success(attempts) => egui::Label::new(
                RichText::new(format!("Success in {} tries!", attempts))
                    .size(24.0)
                    .strong()
                    .color(Color32::DARK_GREEN)
            ),
            Self::Failure => egui::Label::new(
                RichText::new(format!("The correct word was \"{}\".", correct_word))
                    .size(24.0)
                    .strong()
                    .color(Color32::RED)
            )
        }
    }
}

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))]
#[derive(Debug)]
struct WordleCell {
    state: CellState,
    letter: char,
}

impl WordleCell {
    fn keyboard(c: char) -> Self {
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
    word: String,
    #[cfg_attr(feature = "persistence", serde(skip))]
    cells: [[WordleCell; 5]; 6],
    #[cfg_attr(feature = "persistence", serde(skip))]
    next_cell: (usize, usize),
    #[cfg_attr(feature = "persistence", serde(skip))]
    keyboard: ([WordleCell; 10], [WordleCell; 9], [WordleCell; 7]),
    #[cfg_attr(feature = "persistence", serde(skip))]
    keyboard_state: HashMap<char, CellState>,
    #[cfg_attr(feature = "persistence", serde(skip))]
    keyboard_keydown: String,
    #[cfg_attr(feature = "persistence", serde(skip))]
    game_state: GameState,
}

impl Default for WordleApp {
    fn default() -> Self {
        Self {
            word: get_random_word(),
            cells: Default::default(),
            next_cell: (0, 0),
            keyboard: (
                KBD_ROW1.chars().map(|c| WordleCell::keyboard(c)).collect::<Vec<WordleCell>>().try_into().unwrap(),
                KBD_ROW2.chars().map(|c| WordleCell::keyboard(c)).collect::<Vec<WordleCell>>().try_into().unwrap(),
                KBD_ROW3.chars().map(|c| WordleCell::keyboard(c)).collect::<Vec<WordleCell>>().try_into().unwrap(),
            ),
            keyboard_state: LETTERS.chars().zip(std::iter::repeat(CellState::Empty)).collect(),
            keyboard_keydown: String::default(),
            game_state: GameState::Playing,
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
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        #[cfg(feature = "persistence")]
        if let Some(storage) = _storage {
            *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
        }
        // for row in self.cells.iter_mut() {
        //     check_word(row, &self.word);
        // }
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
        let Self {
            word, cells, next_cell,
            keyboard, keyboard_state, keyboard_keydown, game_state
        } = self;

        if matches!(game_state, GameState::Playing) {
            let mut found_char = false;
            for letter in LETTERS.chars() {
                // Should be fine since it should only be an uppercase letter (from LETTERS)
                if (
                    ctx.input().key_released(get_key_from_char(letter).unwrap()) || *keyboard_keydown == letter.to_string()
                ) && next_cell.0 < 6 && next_cell.1 < 5 {
                    cells[next_cell.0][next_cell.1].letter = letter;
                    next_cell.1 += 1;
                    found_char = true;
                    break;
                }
            }
            if !found_char {
                if (
                    ctx.input().key_released(Key::Backspace) || *keyboard_keydown == "DEL"
                ) && next_cell.1 > 0 {
                    next_cell.1 -= 1;
                    cells[next_cell.0][next_cell.1].letter = ' ';
                } else if (
                    ctx.input().key_released(Key::Enter) || *keyboard_keydown == "ENT"
                ) && next_cell.1 >= 5 {
                    if check_word(&mut cells[next_cell.0], word, keyboard, keyboard_state) {
                        if cells[next_cell.0].iter().all(|x| matches!(x.state, CellState::Green)) {
                            *game_state = GameState::Success(next_cell.0 + 1);
                        } else if next_cell.0 == 5 {
                            *game_state = GameState::Failure;
                        }
                        next_cell.0 += 1;
                    } else {
                        for cell in cells[next_cell.0].iter_mut() {
                            cell.letter = ' ';
                        }
                    }
                    next_cell.1 = 0;
                }
            }
        }
        if *keyboard_keydown != "" {
            *keyboard_keydown = String::new();
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
                ui.with_layout(Layout::right_to_left(), |ui| {
                    egui::global_dark_light_mode_switch(ui);
                })
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.vertical_centered(|ui| {
                ui.set_width(ui.available_width());
                // ui.label(
                //     RichText::new(word.clone())
                //         .heading()
                // );
                egui::Frame::none().margin(Margin::symmetric(ui.available_width() / 2.0 - 128.0, 12.0)).show(ui, |ui| {
                    egui::Grid::new("wordle_grid")
                        .spacing((4.0, 4.0))
                        .show(ui, |ui| {
                            for row in cells {
                                for cell in row {
                                    egui::Frame::none()
                                        .fill(match cell.state {
                                            CellState::Empty => Color32::BLACK,
                                            CellState::Gray => Color32::DARK_GRAY,
                                            CellState::Yellow => Color32::from_rgb(181, 159, 59),
                                            CellState::Green => Color32::from_rgb(83, 141, 78),
                                        })
                                        .rounding(6.0)
                                        .show(ui, |ui| {
                                            ui.add_sized(
                                                (48.0, 48.0),
                                                egui::Label::new(
                                                    RichText::new(cell.letter)
                                                        .size(36.0)
                                                        .strong()
                                                        .color(Color32::WHITE)
                                                )
                                            );
                                        });
                                }
                                ui.end_row();
                            }
                        });
                });

                //if matches!(game_state, GameState::Success(_) | GameState::Failure) {
                if true {
                    egui::Frame::none()
                        .margin(Margin {left: 0.0, right: 0.0, top: 12.0, bottom: 24.0})
                        .show(ui, |ui| {
                        ui.add_sized(
                            (400.0, 30.0),
                            game_state.get_state_label(word)
                        );
                    });
                }

                egui::Frame::none().margin(Margin::symmetric(ui.available_width() / 2.0 - 198.0, 0.0)).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = (4.0, 0.0).into();
                        add_keyboard_row(ui, &keyboard.0, keyboard_keydown);
                    });
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = (4.0, 0.0).into();
                        egui::Frame::none().margin(Margin::symmetric(20.0, 0.0)).show(ui, |ui| {
                            add_keyboard_row(ui, &keyboard.1, keyboard_keydown);
                        });
                    });
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = (4.0, 0.0).into();
                        add_keyboard_button(ui, "ENT", keyboard_keydown);
                        add_keyboard_row(ui, &keyboard.2, keyboard_keydown);
                        add_keyboard_button(ui, "DEL", keyboard_keydown);
                    });
                });

                // egui::Grid::new("keyboard_grid")
                //     .spacing((4.0, 4.0))
                //     .show(ui, |ui| {
                //         add_keyboard_row(ui, &keyboard.0);
                //         ui.end_row();
                //         add_keyboard_row(ui, &keyboard.1);
                //         ui.end_row();
                //         add_keyboard_button(ui, "ENT");
                //         add_keyboard_row(ui, &keyboard.2);
                //         add_keyboard_button(ui, "DEL");
                //         ui.end_row();
                //     });
            });
            egui::warn_if_debug_build(ui);
        });
    }
}

fn add_keyboard_row(ui: &mut Ui, row: &[WordleCell], keyboard_keydown: &mut String) {
    for key in row {
        egui::Frame::none()
            .fill(match key.state {
                CellState::Empty => Color32::GRAY,
                CellState::Gray => Color32::DARK_GRAY,
                CellState::Yellow => Color32::from_rgb(181, 159, 59),
                CellState::Green => Color32::from_rgb(83, 141, 78),
            })
            .rounding(6.0)
            //.margin(Margin::same(2.0))
            .show(ui, |ui| {
                if ui.add_sized(
                    (36.0, 48.0),
                    egui::Label::new(
                        RichText::new(key.letter)
                            .size(18.0)
                            .strong()
                            .color(Color32::WHITE)
                    ).sense(Sense::click())
                ).clicked() {
                    if *keyboard_keydown == "" {
                        *keyboard_keydown = key.letter.to_string();
                    }
                }
            });
    }
}

fn add_keyboard_button(ui: &mut Ui, text: &str, keyboard_keydown: &mut String) {
    egui::Frame::none()
        .fill(Color32::GRAY)
        .rounding(6.0)
        //.margin(Margin::same(2.0))
        .show(ui, |ui| {
            if ui.add_sized(
                (56.0, 48.0),
                egui::Label::new(
                    RichText::new(text)
                        .size(18.0)
                        .strong()
                        .color(Color32::WHITE)
                ).sense(Sense::click())
            ).clicked() {
                if *keyboard_keydown == "" {
                    *keyboard_keydown = text.to_string();
                }
            }
        });
}

fn check_word(word: &mut [WordleCell; 5],
              correct: &String,
              keyboard: &mut ([WordleCell; 10], [WordleCell; 9], [WordleCell; 7]),
              keyboard_state: &mut HashMap<char, CellState>) -> bool {
    if !crate::WORD_LIST.contains(&&*word.iter().map(|x| x.letter).collect::<String>()) {
        return false;
    }

    for letter in word.iter_mut() {
        letter.state = CellState::Gray;
        promote_cell_state(keyboard_state.get_mut(&letter.letter).unwrap(), CellState::Gray)
    }

    for (i, correct_letter) in correct.chars().enumerate() {
        if word[i].letter == correct_letter {
            word[i].state = CellState::Green;
            keyboard_state.insert(word[i].letter, CellState::Green);
        } else {
            for letter in word.iter_mut() {
                if letter.letter == correct_letter && matches!(letter.state, CellState::Gray) {
                    letter.state = CellState::Yellow;
                    promote_cell_state(keyboard_state.get_mut(&letter.letter).unwrap(), CellState::Yellow);
                }
            }
        }
    }

    for key in keyboard.0.iter_mut() {
        key.state = keyboard_state.get(&key.letter).unwrap().clone();
    }
    for key in keyboard.1.iter_mut() {
        key.state = keyboard_state.get(&key.letter).unwrap().clone();
    }
    for key in keyboard.2.iter_mut() {
        key.state = keyboard_state.get(&key.letter).unwrap().clone();
    }
    return true;
}

fn promote_cell_state(cell: &mut CellState, state: CellState) {
    match state {
        CellState::Empty => {},
        CellState::Gray => if matches!(cell, CellState::Empty) {
            *cell = state;
        }
        CellState::Yellow => if matches!(cell, CellState::Empty | CellState::Gray) {
            *cell = state;
        }
        CellState::Green => {
            *cell = state;
        }
    }
}

// fn get_random_word() -> String {
//     crate::WORD_LIST[WyRand::new().generate_range(0_usize..crate::WORD_LIST.len())].to_string()
// }
fn get_random_word() -> String {
    crate::WORD_LIST.choose(&mut rand::thread_rng()).unwrap().to_string()
}

fn get_key_from_char(c: char) -> Option<Key> {
    match c {
        'A' => Some(Key::A),
        'B' => Some(Key::B),
        'C' => Some(Key::C),
        'D' => Some(Key::D),
        'E' => Some(Key::E),
        'F' => Some(Key::F),
        'G' => Some(Key::G),
        'H' => Some(Key::H),
        'I' => Some(Key::I),
        'J' => Some(Key::J),
        'K' => Some(Key::K),
        'L' => Some(Key::L),
        'M' => Some(Key::M),
        'N' => Some(Key::N),
        'O' => Some(Key::O),
        'P' => Some(Key::P),
        'Q' => Some(Key::Q),
        'R' => Some(Key::R),
        'S' => Some(Key::S),
        'T' => Some(Key::T),
        'U' => Some(Key::U),
        'V' => Some(Key::V),
        'W' => Some(Key::W),
        'X' => Some(Key::X),
        'Y' => Some(Key::Y),
        'Z' => Some(Key::Z),
        _ => None,
    }
}
