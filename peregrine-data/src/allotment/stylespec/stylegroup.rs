use crate::{allotment::{style::{style::{ContainerAllotmentStyle}}, core::allotmentname::{AllotmentName}}, LeafStyle, LeafRequest};
use super::{styletree::StyleTree};

#[derive(Clone)]
pub struct AllStylesForProgram {
    tree: StyleTree
}

impl AllStylesForProgram {
    pub(crate) fn new(tree: StyleTree) -> AllStylesForProgram {
        AllStylesForProgram {
            tree
        }
    }

    pub(crate) fn get_container(&self, name: &AllotmentName) -> ContainerAllotmentStyle {
        self.tree.lookup_container(name)
    }

    pub(crate) fn get_leaf(&self, leaf: &LeafRequest) -> LeafStyle {
        self.tree.lookup_leaf(leaf.name())
    }
}
