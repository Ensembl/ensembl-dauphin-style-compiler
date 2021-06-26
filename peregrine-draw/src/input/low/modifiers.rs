use std::{collections::HashSet, ops::{BitAnd, Sub}};
#[derive(Debug,Clone)]
pub struct KeyboardModifiers {
    pub shift: bool,
    pub control: bool,
    pub alt: bool
}

impl KeyboardModifiers {
    pub(super) fn new(shift: bool, control: bool, alt: bool) -> KeyboardModifiers {
        KeyboardModifiers { shift, control, alt }
    }

    fn empty(&self) -> bool {
        !(self.shift || self.control || self.alt)
    }
}

impl Sub for &KeyboardModifiers {
    type Output = KeyboardModifiers;

    fn sub(self, other: &KeyboardModifiers) -> KeyboardModifiers {
        KeyboardModifiers {
            shift: self.shift && !other.shift,
            control: self.control && !other.control,
            alt: self.alt && !other.alt
        }
    }
}

impl BitAnd for &KeyboardModifiers {
    type Output = KeyboardModifiers;

    fn bitand(self, other: &KeyboardModifiers) -> KeyboardModifiers {
        KeyboardModifiers {
            shift: self.shift && other.shift,
            control: self.control && other.control,
            alt: self.alt && other.alt
        }
    }    
}

#[derive(Debug,Clone)]
pub struct Modifiers {
    keyboard: KeyboardModifiers,
    artificial: HashSet<String>
}

impl Modifiers {
    pub fn new(keyboard: KeyboardModifiers, artificial: &[String]) -> Modifiers {
        Modifiers {
            keyboard,
            artificial: artificial.iter().cloned().collect()
        }
    }

    pub(super) fn update_keyboard_modifiers(&mut self, modifiers: KeyboardModifiers) {
        self.keyboard = modifiers;
    }

    pub fn set_artificial(&mut self, name: &str, start: bool) {
        if start {
            self.artificial.insert(name.to_string());
        } else {
            self.artificial.remove(name);
        }
    }
}


#[derive(Debug,Clone)]
pub struct ModifiersPattern {
    required: Modifiers,
    prohibited: Modifiers
}

impl ModifiersPattern {
    pub(super) fn new(required: Modifiers,prohibited: Modifiers) -> ModifiersPattern {
        ModifiersPattern { required, prohibited }
    }

    pub(super) fn is_match(&self, modifiers: &Modifiers) -> bool {
        if !(&self.required.keyboard - &modifiers.keyboard).empty() { return false; }
        if !(&modifiers.keyboard & &self.prohibited.keyboard).empty() { return false; }
        if (&self.required.artificial - &modifiers.artificial).len() != 0 { return false; }
        if (&modifiers.artificial & &self.prohibited.artificial).len() != 0 { return false; }
        true
    }
}
