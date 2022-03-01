/*
use std::{sync::{Arc, Mutex}, collections::HashMap};

use peregrine_toolkit::{lock, puzzle::{PuzzleBuilder, PuzzleValueHolder, PuzzlePiece}};

use crate::{allotment::{core::arbitrator::BpPxConverter, style::style::{Padding, LeafCommonStyle}, boxes::{leaf::{FloatingLeaf}, stacker::Stacker, overlay::Overlay, bumper::Bumper}}, CoordinateSystem, CoordinateSystemVariety};

pub enum LeafAllotmentType {
    Leaf
}

pub enum ContainerAllotmentType {
    Stack,
    Overlay,
    Bumper
}

impl ContainerAllotmentType {
    fn build(spec: &HashMap<String,String>) -> ContainerAllotmentType {
        let type_str = spec.get("type").map(|x| x.as_str());
        match type_str {
            Some("overlay") => ContainerAllotmentType::Overlay,
            Some("bumper") => ContainerAllotmentType::Bumper,
            _ => ContainerAllotmentType::Stack
        }    
    }
}

impl LeafAllotmentType {
    fn build(spec: &HashMap<String,String>) -> LeafAllotmentType {
        LeafAllotmentType::Leaf
    }
}

pub struct LeafAllotmentStyle {
    allot_type: LeafAllotmentType,
    leaf: LeafCommonStyle,
}

impl LeafAllotmentStyle {
    fn empty() -> LeafAllotmentStyle {
        LeafAllotmentStyle {
            allot_type: LeafAllotmentType::Leaf,
            leaf: LeafCommonStyle::default()
        }
    }

    fn build(spec: &HashMap<String,String>) -> LeafAllotmentStyle {
        let allot_type = LeafAllotmentType::build(spec);
        let leaf = LeafCommonStyle::build(spec);
        LeafAllotmentStyle { allot_type, leaf }
    }
}

pub struct ContainerAllotmentStyle {
    allot_type: ContainerAllotmentType,
    padding: Padding
}

impl ContainerAllotmentStyle {
    fn empty() -> ContainerAllotmentStyle {
        ContainerAllotmentStyle {
            allot_type: ContainerAllotmentType::Stack,
            padding: Padding::empty()
        }
    }

    fn build(spec: &HashMap<String,String>) -> ContainerAllotmentStyle {
        let allot_type = ContainerAllotmentType::build(spec);
        ContainerAllotmentStyle { allot_type, padding: Padding::build(spec) }
    }
}

// XXX excessive cloning: does this really need to be mutex?
pub struct AllotmentStyleGroup {
    container_style: Arc<Mutex<HashMap<Vec<String>,Arc<ContainerAllotmentStyle>>>>,
    container_empty: Arc<ContainerAllotmentStyle>,
    leaf_style: Arc<Mutex<HashMap<Vec<String>,Arc<LeafAllotmentStyle>>>>,
    leaf_empty: Arc<LeafAllotmentStyle>
}

impl AllotmentStyleGroup {
    pub fn empty() -> AllotmentStyleGroup {
        AllotmentStyleGroup {
            container_style: Arc::new(Mutex::new(HashMap::new())),
            container_empty: Arc::new(ContainerAllotmentStyle::empty()),
            leaf_style: Arc::new(Mutex::new(HashMap::new())),
            leaf_empty: Arc::new(LeafAllotmentStyle::empty())
        }
    }

    fn get_container(&self, name: &AllotmentNamePart) -> Arc<ContainerAllotmentStyle> {
        lock!(self.container_style).get(&name.sequence().to_vec()).cloned().unwrap_or(self.container_empty.clone())
    }

    fn get_leaf(&self, name: &AllotmentNamePart) -> Arc<LeafAllotmentStyle> {
        lock!(self.leaf_style).get(&name.sequence().to_vec()).cloned().unwrap_or(self.leaf_empty.clone())
    }
}

#[derive(Clone,Hash,PartialEq,Eq)]
pub struct AllotmentName {
    name: Arc<Vec<String>>
}

impl AllotmentName {
    fn new(spec: &str) -> AllotmentName {
        let name = spec.split("/").map(|x| x.to_string()).collect();
        AllotmentName {
            name: Arc::new(name)
        }
    }
}

#[derive(Clone)]
pub struct AllotmentNamePart {
    name: AllotmentName,
    end: usize
}

impl AllotmentNamePart {
    fn new(name: AllotmentName) -> AllotmentNamePart {
        AllotmentNamePart {
            end: name.name.len(), name
        }
    }

    fn sequence(&self) -> &[String] { &self.name.name[0..self.end] }

    fn pop(&self) -> Option<(&str,AllotmentNamePart)> {
        if self.end > 0 {
            let mut part = self.clone();
            part.end -= 1;
            Some((&self.name.name[self.end-1],part))
        } else {
            None
        }
    }
}

pub(crate) enum ContainerHolder {
    Stack(Stacker),
    Overlay(Overlay),
    Bumper(Bumper)
}

#[derive(Clone)]
pub enum LeafHolder {
    Leaf(FloatingLeaf)
}

pub struct AllotmentStyler {
    puzzle: PuzzleBuilder,
    converter: Arc<BpPxConverter>,
    leafs_made: HashMap<Vec<String>,LeafHolder>,
    containers_made: HashMap<Vec<String>,ContainerHolder>,
    anchors: HashMap<String,PuzzlePiece<f64>>,
    dustbin: FloatingLeaf
}

impl AllotmentStyler {
    fn new_container(&self, name: &AllotmentNamePart, styles: &AllotmentStyleGroup) -> ContainerHolder {
        let style = styles.get_container(name);
        match style.allot_type {
            ContainerAllotmentType::Stack => {
                ContainerHolder::Stack(Stacker::new(&self.puzzle,&style.padding))
            },
            ContainerAllotmentType::Overlay => {
                ContainerHolder::Overlay(Overlay::new(&self.puzzle,&style.padding))
            },
            ContainerAllotmentType::Bumper => {
                ContainerHolder::Bumper(Bumper::new(&self.puzzle,&style.padding))
            }
        }
    }

    fn new_floating_leaf(&self, container: &mut ContainerHolder,  name: &AllotmentNamePart, styles: &AllotmentStyleGroup) -> FloatingLeaf {
        let style = styles.get_leaf(name);
        let child = FloatingLeaf::new(&self.puzzle,&self.converter,&style.leaf);
        match container {
            ContainerHolder::Stack(stack) => {
                stack.add_child(&child);
            },
            ContainerHolder::Overlay(overlay) => {
                overlay.add_child(&child);
            },
            ContainerHolder::Bumper(bumper) => {
                bumper.add_child(&child);
            }
        }
        child
    }

    fn new_leaf(&self, name: &AllotmentNamePart, styles: &AllotmentStyleGroup) -> LeafHolder {
        if let Some((_,rest)) = name.pop() {
            let mut container = self.new_container(&rest,styles);
            LeafHolder::Leaf(self.new_floating_leaf(&mut container,name,styles))
        } else {
            LeafHolder::Leaf(self.dustbin.clone())
        }
    }

    fn try_new_leaf(&mut self, name: &AllotmentNamePart, styles: &AllotmentStyleGroup) -> LeafHolder {
        let sequence = name.sequence().to_vec();
        if let Some(leaf) = self.leafs_made.get(&sequence) {
            leaf.clone()
        } else {
            let out = self.new_leaf(name,styles);
            self.leafs_made.insert(sequence,out.clone());
            out
        }
    }

    fn new(puzzle: &PuzzleBuilder, converter: &Arc<BpPxConverter>, names: Vec<AllotmentName>, styles: &AllotmentStyleGroup) -> AllotmentStyler {
        let mut styler = AllotmentStyler {
            leafs_made: HashMap::new(),
            containers_made: HashMap::new(),
            anchors: HashMap::new(),
            puzzle: puzzle.clone(),
            converter: converter.clone(),
            dustbin: FloatingLeaf::new(puzzle,converter,&LeafCommonStyle::dustbin())
        };
        for name in names {
            let parts = AllotmentNamePart::new(name);
            styler.try_new_leaf(&parts,styles);
        }
        styler
    }
}
*/