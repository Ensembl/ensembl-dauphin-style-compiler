use std::{sync::{Mutex, Arc}, collections::HashMap, borrow::Borrow};

use peregrine_toolkit::lock;

use crate::allotment::{boxes::leaf, style::{allotmentname::{AllotmentNamePart, AllotmentName}, style::{ContainerAllotmentStyle, LeafAllotmentStyle}, pendingleaf::PendingLeaf}};

use super::styletree::StyleTree;

#[derive(Clone)]
pub struct AllotmentStyleGroup {
    tree: Arc<StyleTree>
}

impl AllotmentStyleGroup {
    pub fn empty() -> AllotmentStyleGroup {
        AllotmentStyleGroup {
            tree: Arc::new(StyleTree::empty())
        }
    }

    pub fn new(tree: StyleTree) -> AllotmentStyleGroup {
        AllotmentStyleGroup {
            tree: Arc::new(tree)
        }
    }

    pub(crate) fn get_container(&self, name: &AllotmentNamePart) -> &ContainerAllotmentStyle {
        self.tree.get_container(name)
    }

    pub(crate) fn get_leaf(&self, name: &AllotmentNamePart) -> &LeafAllotmentStyle {
        self.tree.get_leaf(name)
    }

    pub(crate) fn get_pending_leaf(&self, leaf: &PendingLeaf) -> &LeafAllotmentStyle {
        self.get_leaf(&AllotmentNamePart::new(leaf.name().clone()))
    }
}
