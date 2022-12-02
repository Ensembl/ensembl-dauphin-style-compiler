use std::{sync::{Arc}};
use crate::{allotment::{style::{style::{ContainerAllotmentStyle}}, core::allotmentname::{AllotmentNamePart, AllotmentName}}, LeafStyle, LeafRequest};
use super::{styletree::StyleTree, specifiedstyle::{InheritableStyle}};

#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(Clone)]
pub struct AllStylesForProgram {
    tree: Arc<StyleTree>
}

impl AllStylesForProgram {
    pub fn new(tree: StyleTree) -> AllStylesForProgram {
        AllStylesForProgram {
            tree: Arc::new(tree)
        }
    }

    pub(crate) fn get_container(&self, name: &AllotmentName) -> &ContainerAllotmentStyle {
        let part = AllotmentNamePart::new(name.clone());
        self.tree.get_container(&part)
    }

    pub(crate) fn get_leaf(&self, leaf: &LeafRequest) -> LeafStyle {
        let mut inherit = InheritableStyle::empty();
        for name in AllotmentNamePart::new(leaf.name().clone()).iter_prefixes() {
            inherit.override_style(&self.tree.get_container(&name).leaf);
        }
        let style = self.tree.get_leaf(&AllotmentNamePart::new(leaf.name().clone()));
        inherit.override_style(&style.leaf);
        inherit.make(&style)
    }
}
