use std::{sync::{Arc, Mutex}, hash::{Hash, Hasher}};
use crate::lock;

pub trait PuzzleDependency {
    fn index(&self) -> Option<usize> { self.index }
}

#[cfg_attr(test,derive(Debug))]
#[derive(Clone)]
pub struct RealPuzzleDependency {
    index: Option<usize>,
    #[cfg(debug_assertions)]
    name: Arc<Mutex<String>>
}

impl PartialEq for RealPuzzleDependency {
    fn eq(&self, other: &Self) -> bool { self.index == other.index }
}

impl Eq for RealPuzzleDependency {}

impl Hash for RealPuzzleDependency {
    fn hash<H: Hasher>(&self, state: &mut H) { self.index.hash(state); }
}

impl RealPuzzleDependency {
    pub(super) fn new(index: usize) -> RealPuzzleDependency {
        RealPuzzleDependency {
            index: Some(index),
            #[cfg(debug_assertions)]
            name: Arc::new(Mutex::new("".to_string()))
         }
    }

    #[cfg(debug_assertions)]
    pub fn name(&self) -> String { lock!(self.name).clone() }

    #[cfg(debug_assertions)]
    pub fn set_name(&mut self, name: &str) { *lock!(self.name) = name.to_string(); }

    pub(super) fn none() -> RealPuzzleDependency {
        RealPuzzleDependency {
             index: None,
             #[cfg(debug_assertions)]
             name: Arc::new(Mutex::new("".to_string()))
        }
    }
}

impl PuzzleDependency for RealPuzzleDependency {
    fn index(&self) -> Option<usize> { self.index }
}
