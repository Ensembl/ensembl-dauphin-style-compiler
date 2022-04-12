use std::{collections::HashMap, sync::{Arc, Mutex}};

use peregrine_toolkit::{lock, puzzle::{constant, DelayedSetter,StaticValue, promise_delayed}};

use crate::{allotment::{boxes::root::{Root}, style::style::Indent}};

use super::playingfield::PlayingFieldPieces;

struct Datum {
    piece: StaticValue<f64>,
    piece_setter: DelayedSetter<'static,'static,f64>
}

impl Datum {
    fn new() -> Datum {
        let (piece_setter,piece) = promise_delayed();
        Datum { piece, piece_setter }
    }

    fn set(&mut self, value: &StaticValue<f64>) {
        let value = value.clone();
        self.piece_setter.set(value);
    }

    fn get(&self) -> StaticValue<f64> {
        self.piece.clone()
    }
}

#[derive(Clone)]
pub struct Aligner {
    playing_field: PlayingFieldPieces,
    datums: Arc<Mutex<HashMap<String,Datum>>>
}

impl Aligner {
    pub(crate) fn new(root: &Root) -> Aligner {
        Aligner {
            playing_field: root.playing_field_pieces(),
            datums: Arc::new(Mutex::new(HashMap::new()))
        }
    }

    pub(crate) fn get(&self, indent: &Indent) -> StaticValue<f64> {
        match match indent {
            Indent::Top => Some(&self.playing_field.top),
            Indent::Left => Some(&self.playing_field.left),
            Indent::Bottom => Some(&self.playing_field.bottom),
            Indent::Right => Some(&self.playing_field.right),
            _ => None
        } {
            Some(piece) => { return piece.clone() },
            None => {}
        }
        match match indent {
            Indent::Datum(datum) => Some(self.get_datum(datum)),
            _ => None
        } {
            Some(value) => { return value.clone() },
            None => {}
        }
        constant(0.)
    }
    
    pub(crate) fn set_datum(&self, datum: &str, value: &StaticValue<f64>) {
        lock!(self.datums).entry(datum.to_string()).or_insert_with(move || Datum::new()).set(value);
    }

    pub(crate) fn get_datum(&self,  datum: &str) -> StaticValue<f64> {
        lock!(self.datums).entry(datum.to_string()).or_insert_with(|| Datum::new()).get()
    }
}
