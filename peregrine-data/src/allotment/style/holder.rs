use std::sync::Arc;
use crate::{allotment::{boxes::{stacker::Stacker, overlay::Overlay, bumper::Bumper}, boxes::{boxtraits::{Stackable, Transformable }, leaf::FloatingLeaf, root::{Root}}}, DataMessage};

#[derive(Clone)]
pub enum ContainerHolder {
    Root(Root),
    Stack(Stacker),
    Overlay(Overlay),
    Bumper(Bumper),
}

impl ContainerHolder {
    pub(super) fn add_leaf(&mut self, child: &LeafHolder) {
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

    pub(super) fn stackable(&self) -> Result<&dyn Stackable,DataMessage> {
        Ok(match self {
            ContainerHolder::Root(_) => { return Err(DataMessage::BadBoxStack(format!("cannot add root as child"))); }
            ContainerHolder::Stack(x) => x,
            ContainerHolder::Overlay(x) => x,
            ContainerHolder::Bumper(x) => x
        })
    }

    pub(super) fn add_container(&mut self, container: &ContainerHolder) -> Result<(),DataMessage> {
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
    pub(super) fn into_tranfsormable(self) -> Arc<dyn Transformable> {
        match self {
            LeafHolder::Leaf(leaf) => Arc::new(leaf)
        }
    }
}
