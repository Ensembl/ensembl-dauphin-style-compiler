use std::{sync::{Arc}};

use crate::{allotment::{style::{style::{ContainerAllotmentStyle, LeafAllotmentStyle, LeafInheritStyle}}, core::allotmentname::AllotmentNamePart}, LeafCommonStyle, LeafRequest};

use super::styletree::StyleTree;

#[cfg_attr(debug_assertions,derive(Debug))]
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

    pub(crate) fn get_common_leaf(&self, leaf: &LeafRequest) -> LeafCommonStyle {
        let mut inherit = LeafInheritStyle::empty();
        for name in AllotmentNamePart::new(leaf.name().clone()).iter_prefixes() {
            inherit.override_style(&self.get_container(&name).leaf);
        }
        let style = self.get_leaf(&AllotmentNamePart::new(leaf.name().clone()));
        inherit.override_style(&style.leaf);
        inherit.make(&style)
    }
}
