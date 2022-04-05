use crate::app::{CellState, GameState, WordleCell};
use crate::WordleApp;
use eframe::egui::{Key, Ui};
use rand::seq::SliceRandom;
use std::collections::HashMap;

const LETTERS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const KBD_ROW1: &str = "QWERTYUIOP";
const KBD_ROW2: &str = "ASDFGHJKL";
const KBD_ROW3: &str = "ZXCVBNM";

impl WordleApp {
    pub(crate) fn handle_keys(&mut self, ctx: &Ui) {
        let mut found_char = false;
        // Check for pressed letter keys
        for letter in LETTERS.chars() {
            // Should be fine since it should only be an uppercase letter (from LETTERS)
            if (ctx.input().key_released(get_key_from_char(letter).unwrap())
                || self.kbd_keydown == letter.to_string())
                && self.next_cell.0 < 6
                && self.next_cell.1 < 5
            {
                // Write the pressed letter and advance the cursor
                self.cells[self.next_cell.0][self.next_cell.1].letter = letter;
                self.next_cell.1 += 1;
                found_char = true;
                break;
            }
        }
        // Check for pressed non-letter keys
        if !found_char {
            if (ctx.input().key_released(Key::Backspace) || self.kbd_keydown == "DEL")
                && self.next_cell.1 > 0
            {
                // Delete letter
                self.next_cell.1 -= 1;
                self.cells[self.next_cell.0][self.next_cell.1].letter = ' ';
            } else if (ctx.input().key_released(Key::Enter) || self.kbd_keydown == "ENT")
                && self.next_cell.1 >= 5
            {
                // Check a completed word
                if check_word(
                    &mut self.cells[self.next_cell.0],
                    &self.word,
                    &mut self.keyboard,
                    &mut self.keyboard_state,
                ) {
                    if self.cells[self.next_cell.0]
                        .iter()
                        .all(|x| matches!(x.state, CellState::Green))
                    {
                        self.game_state = GameState::Success(self.next_cell.0 + 1);
                    } else if self.next_cell.0 == 5 {
                        self.game_state = GameState::Failure;
                    }
                    self.next_cell.0 += 1;
                } else {
                    for cell in self.cells[self.next_cell.0].iter_mut() {
                        cell.letter = ' ';
                    }
                }
                self.next_cell.1 = 0;
            }
        }
    }

    pub(crate) fn reset_random_word(&mut self) {
        self.word = get_random_word();
        self.reset();
    }

    #[rustfmt::skip] // rustfmt wants to split the keyboard initialization into loads of tiny lines
    pub(crate) fn reset(&mut self) {
        self.cells = Default::default();
        self.next_cell = (0, 0);
        self.keyboard = (
            KBD_ROW1.chars().map(WordleCell::keyboard)
                .collect::<Vec<WordleCell>>().try_into().unwrap(),
            KBD_ROW2.chars().map(WordleCell::keyboard)
                .collect::<Vec<WordleCell>>().try_into().unwrap(),
            KBD_ROW3.chars().map(WordleCell::keyboard)
                .collect::<Vec<WordleCell>>().try_into().unwrap(),
        );
        self.keyboard_state = LETTERS.chars().zip(std::iter::repeat(CellState::Empty)).collect();
        self.game_state = GameState::Playing;
    }

    pub(crate) fn with_args(args: Args) -> Self {
        let mut app = Self::default();
        app.args = args;
        app
    }
}

pub(crate) fn promote_cell_state(cell: &mut CellState, state: CellState) {
    match state {
        CellState::Empty => {}
        CellState::Gray => {
            if matches!(cell, CellState::Empty) {
                *cell = state;
            }
        }
        CellState::Yellow => {
            if matches!(cell, CellState::Empty | CellState::Gray) {
                *cell = state;
            }
        }
        CellState::Green => {
            *cell = state;
        }
    }
}

// fn get_random_word() -> String {
//     crate::WORD_LIST[WyRand::new().generate_range(0_usize..crate::WORD_LIST.len())].to_string()
// }
pub(crate) fn get_random_word() -> String {
    crate::WORD_LIST.choose(&mut rand::thread_rng()).unwrap().to_string()
}

pub(crate) fn get_key_from_char(c: char) -> Option<Key> {
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

fn check_word(
    word: &mut [WordleCell; 5],
    correct: &str,
    keyboard: &mut ([WordleCell; 10], [WordleCell; 9], [WordleCell; 7]),
    keyboard_state: &mut HashMap<char, CellState>,
) -> bool {
    // Verify word is in list
    if !crate::WORD_LIST.contains(&&*word.iter().map(|x| x.letter).collect::<String>()) {
        return false;
    }

    // Start by assuming no letters match
    for letter in word.iter_mut() {
        letter.state = CellState::Gray;
        promote_cell_state(keyboard_state.get_mut(&letter.letter).unwrap(), CellState::Gray)
    }

    for (i, correct_letter) in correct.chars().enumerate() {
        if word[i].letter == correct_letter {
            // If a letter matches exactly, turn it green
            word[i].state = CellState::Green;
            keyboard_state.insert(word[i].letter, CellState::Green);
        } else {
            for letter in word.iter_mut() {
                // Look for a matching cell somewhere else in the word. If one is found and it's
                // gray, turn it yellow.
                if letter.letter == correct_letter && matches!(letter.state, CellState::Gray) {
                    letter.state = CellState::Yellow;
                    promote_cell_state(
                        keyboard_state.get_mut(&letter.letter).unwrap(),
                        CellState::Yellow,
                    );
                    break;
                }
            }
        }
    }

    // Update the keyboard keys based on the their state from the alphabet
    for key in keyboard.0.iter_mut() {
        key.state = keyboard_state.get(&key.letter).unwrap().clone();
    }
    for key in keyboard.1.iter_mut() {
        key.state = keyboard_state.get(&key.letter).unwrap().clone();
    }
    for key in keyboard.2.iter_mut() {
        key.state = keyboard_state.get(&key.letter).unwrap().clone();
    }

    true
}

#[derive(Default, Clone)]
pub(crate) struct Args {
    pub(crate) word: Option<String>,
}

pub(crate) fn encode(s: String) -> String {
    let mut out = s.to_uppercase();
    for _ in 0..5 {
        out = base64::encode_config(out, base64::URL_SAFE_NO_PAD);
    }
    out
}

pub(crate) fn decode(s: String) -> Result<String, base64::DecodeError> {
    let mut out = s;
    for _ in 0..5 {
        out = String::from_utf8_lossy(
            &*base64::decode_config(out, base64::URL_SAFE_NO_PAD)?).into();
    }
    Ok(out.to_uppercase())
}
