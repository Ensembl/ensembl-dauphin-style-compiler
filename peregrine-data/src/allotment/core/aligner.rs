use std::{collections::HashMap, sync::{Arc, Mutex}};

use peregrine_toolkit::{puzzle::{PuzzleValueHolder, ConstantPuzzlePiece, PuzzlePiece, PuzzleBuilder, PuzzleValue}, lock};

use crate::{allotment::{boxes::root::{PlayingFieldPieces, Root}, style::style::Indent}, DataMessage};

struct Datum {
    piece: PuzzlePiece<f64>
}

impl Datum {
    fn new(puzzle: &PuzzleBuilder) -> Datum {
        Datum { piece: puzzle.new_piece_default(0.) }
    }

    fn set(&mut self, value: &PuzzleValueHolder<f64>) {
        let value = value.clone();
        self.piece.add_solver(&[value.dependency()], move |solution| {
            Some(*value.get(solution))
        });
    }

    fn get(&self) -> PuzzleValueHolder<f64> {
        PuzzleValueHolder::new(self.piece.clone())
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

    pub(crate) fn get(&self, puzzle: &PuzzleBuilder, indent: &Indent) -> PuzzleValueHolder<f64> {
        match match indent {
            Indent::Top => Some(&self.playing_field.top),
            Indent::Left => Some(&self.playing_field.left),
            Indent::Bottom => Some(&self.playing_field.bottom),
            Indent::Right => Some(&self.playing_field.right),
            _ => None
        } {
            Some(piece) => { return PuzzleValueHolder::new(piece.clone()) },
            None => {}
        }
        match match indent {
            Indent::Datum(datum) => Some(self.get_datum(puzzle,datum)),
            _ => None
        } {
            Some(value) => { return value.clone() },
            None => {}
        }
        PuzzleValueHolder::new(ConstantPuzzlePiece::new(0.))
    }
    
    pub(crate) fn set_datum(&self, puzzle: &PuzzleBuilder, datum: &str, value: &PuzzleValueHolder<f64>) {
        lock!(self.datums).entry(datum.to_string()).or_insert_with(|| Datum::new(puzzle)).set(value);
    }

    pub(crate) fn get_datum(&self,  puzzle: &PuzzleBuilder, datum: &str) -> PuzzleValueHolder<f64> {
        lock!(self.datums).entry(datum.to_string()).or_insert_with(|| Datum::new(puzzle)).get()
    }
}
