use std::sync::Arc;
use peregrine_toolkit::error::Error;
use crate::{allotment::{boxes::{stacker::Stacker, overlay::Overlay, bumper::Bumper}, boxes::{leaf::FloatingLeaf, root::{Root}}, core::boxtraits::{Stackable, Transformable}}};

#[derive(Clone)]
pub enum ContainerHolder {
    Root(Root),
    Stack(Stacker),
    Overlay(Overlay),
    Bumper(Bumper),
}

impl ContainerHolder {
    pub(crate) fn add_leaf(&mut self, child: &LeafHolder) {
        match (self,child) {
            (ContainerHolder::Root(root),LeafHolder::Leaf(leaf)) => {
                root.add_child(leaf);
            },
            (ContainerHolder::Stack(stack),LeafHolder::Leaf(leaf)) => {
                stack.add_child(leaf);
            },
            (ContainerHolder::Overlay(overlay),LeafHolder::Leaf(leaf)) => {
                overlay.add_child(leaf);
            },
            (ContainerHolder::Bumper(bumper),LeafHolder::Leaf(leaf)) => {
                bumper.add_child(leaf);
            }
        }
    }

    pub(super) fn stackable(&self) -> Result<&dyn Stackable,Error> {
        Ok(match self {
            ContainerHolder::Root(_) => { return Err(Error::operr(&format!("bad box stack: cannot add root as child"))); }
            ContainerHolder::Stack(x) => x,
            ContainerHolder::Overlay(x) => x,
            ContainerHolder::Bumper(x) => x
        })
    }

    pub(crate) fn add_container(&mut self, container: &ContainerHolder) -> Result<(),Error> {
        match self {
            ContainerHolder::Bumper(parent) => {
                parent.add_child(container.stackable()?);
            },
            ContainerHolder::Root(parent) => {
                parent.add_child(container.stackable()?);
            },
            ContainerHolder::Overlay(parent) => {
                parent.add_child(container.stackable()?);            
            },
            ContainerHolder::Stack(parent) => {
                parent.add_child(container.stackable()?);
            }
        }
        Ok(())
    }
}

#[derive(Clone)]
pub enum LeafHolder {
    Leaf(FloatingLeaf)
}

impl LeafHolder {
    pub(crate) fn into_tranfsormable(self) -> Arc<dyn Transformable> {
        match self {
            LeafHolder::Leaf(leaf) => Arc::new(leaf)
        }
    }
}
