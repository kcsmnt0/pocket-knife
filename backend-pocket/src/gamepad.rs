use enumset::{EnumSet, EnumSetType};
use litex_pac::{APF_INPUT, Peripherals};
use slint::{platform::Key, SharedString};
use strum::EnumIter;

#[derive(EnumSetType, EnumIter, Debug)]
#[enumset(repr = "u16")]
pub enum Button {
    Up, Down, Left, Right,
    A, B, X, Y,
    L1, R1, L2, R2,
    L3, R3, Select, Start,
}

#[derive(Debug, PartialEq, Eq)]
pub enum State {
    Idle, Pressed, Released, Held,
}

#[derive(Debug, Default)]
pub struct Gamepad {
    last: EnumSet<Button>,
    current: EnumSet<Button>,
}

impl Button {
    pub fn key_text(self) -> SharedString {
        match self {
            Button::Up => SharedString::from(Key::UpArrow),
            Button::Down => SharedString::from(Key::DownArrow),
            Button::Left => SharedString::from(Key::LeftArrow),
            Button::Right => SharedString::from(Key::RightArrow),
            Button::A => SharedString::from("a"),
            Button::B => SharedString::from("b"),
            Button::X => SharedString::from("x"),
            Button::Y => SharedString::from("y"),
            Button::L1 => SharedString::from("l"),
            Button::R1 => SharedString::from("r"),
            Button::L2 => SharedString::from(Key::PageUp),
            Button::R2 => SharedString::from(Key::PageDown),
            Button::L3 => SharedString::from(Key::Home),
            Button::R3 => SharedString::from(Key::End),
            Button::Select => SharedString::from(Key::Escape),
            Button::Start => SharedString::from(Key::Return),
        }
    }
}

impl Gamepad {
    pub fn update(&mut self) {
        let input = unsafe { Peripherals::steal().APF_INPUT };
        let new = EnumSet::from_repr(input.cont1_key.read().bits() as u16);
        self.last = self.current;
        self.current = new;
    }

    pub fn state(&self, button: Button) -> State {
        match (self.last.contains(button), self.current.contains(button)) {
            (false, false) => State::Idle,
            (false, true) => State::Pressed,
            (true, false) => State::Released,
            (true, true) => State::Held,
        }
    }
}
