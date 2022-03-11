use std::sync::Arc;

use crate::allotment::{boxes::{stacker::Stacker, overlay::Overlay, bumper::Bumper}, boxes::{boxtraits::{Stackable, Transformable}, leaf::FloatingLeaf, root::Root}};

#[derive(Clone)]
pub enum ContainerHolder {
    Root(Root),
    Stack(Stacker),
    Overlay(Overlay),
    Bumper(Bumper)
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

    fn get_stackable(&self) -> Result<&dyn Stackable,String> {
        match self {
            ContainerHolder::Root(_) => Err("root is not stackable".to_string()),
            ContainerHolder::Stack(x) => Ok(x),
            ContainerHolder::Overlay(x) => Ok(x),
            ContainerHolder::Bumper(x) => Ok(x)
        }
    }

    fn get_stackable_ranged(&self) -> Result<(),String> {
        match self {
            ContainerHolder::Root(x) => Err("root is not ranged".to_string()),
            ContainerHolder::Stack(x) => Err("stack is not ranged".to_string()),
            ContainerHolder::Overlay(x) => Err("oferlay is not ranged".to_string()),
            ContainerHolder::Bumper(x) => Err("bumper is not ranged".to_string()),
        }
    }

    pub(super) fn add_container(&mut self, container: &ContainerHolder) -> Result<(),String> {
        match self {
            ContainerHolder::Bumper(_) => {
                return Err("container is not ranged".to_string());
            },
            ContainerHolder::Root(parent) => {
                parent.add_child(container.get_stackable()?);
            },
            ContainerHolder::Overlay(parent) => {
                parent.add_child(container.get_stackable()?);            
            },
            ContainerHolder::Stack(parent) => {
                parent.add_child(container.get_stackable()?);
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
