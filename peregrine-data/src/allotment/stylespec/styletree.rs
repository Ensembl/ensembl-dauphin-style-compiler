use std::{collections::HashMap};

use crate::allotment::{style::{style::{ContainerAllotmentStyle}}, core::allotmentname::AllotmentNamePart};

use super::{styletreebuilder::StyleTreeBuilder, specifiedstyle::SpecifiedStyle};

/* These trees go leaf-to-root!, to make ** possible. It's best to think of these trees as enumerating all paths which
 * have distinct answers to questions of style, as far as they need to go to distinguish between them. Also, think of
 * the builder trees as largely the writable parallels of these rad-only trees.
 * 
 * In this context implementing "** / X" means finding X and then 1. adding this style wherever a search might terminate
 * successfully and 2. stop things failing, but replace them with this. The only place a search can terminate is at a
 * completely matching node or at a node with no "None" child. So to add any's we first traverse through X and then flood
 * visit the rest of the extant tree.  Each node gets the relevant properties. Nodes without a None child (any) also get
 * one, with the all flag set to true so that all children get the indicated properties.
 */

/* NB: In StyleTreeNodes, None means "other", ie all * properties which are not overridden are also propagated to extant
 * not-None leaves.
 */

#[cfg_attr(debug_assertions,derive(Debug))]
struct Styles {
    container: ContainerAllotmentStyle,
    leaf: SpecifiedStyle,
}

#[cfg_attr(debug_assertions,derive(Debug))]
pub(super) struct StyleTreeNode {
    here: Styles,
    all: bool,
    children: HashMap<Option<String>,StyleTreeNode>
}

impl StyleTreeNode {
    pub(super) fn new(container: ContainerAllotmentStyle, leaf: SpecifiedStyle, all: bool) -> StyleTreeNode {
        StyleTreeNode {
            here: Styles { container, leaf },
            children: HashMap::new(),
            all
        }
    }

    /* only used during building */
    pub(super) fn add(&mut self, name: Option<&String>, node: StyleTreeNode) {
        self.children.insert(name.cloned(),node);
    }

    fn get(&self, name: &AllotmentNamePart) -> Option<&StyleTreeNode> {
        if let Some((tail,head)) = name.pop() {
            if let Some(child) = self.children.get(&Some(tail)) {
                child.get(&head)
            } else if let Some(other) = self.children.get(&None) {
                if self.all { Some(other) } else { other.get(&head) }
            } else {
                None
            }
        } else {
            Some(self)
        }
    }

    fn get_container(&self, name: &AllotmentNamePart) -> Option<&ContainerAllotmentStyle> {
        self.get(name).map(|x| &x.here.container)
    }

    fn get_leaf(&self, name: &AllotmentNamePart) -> Option<&SpecifiedStyle> {
        self.get(name).map(|x| &x.here.leaf)
    }
}

#[cfg_attr(debug_assertions,derive(Debug))]
pub struct StyleTree {
    root: StyleTreeNode,
    empty_container: ContainerAllotmentStyle,
    empty_leaf: SpecifiedStyle
}

impl StyleTree {
    pub fn new(mut builder: StyleTreeBuilder) -> StyleTree { builder.build() }

    pub fn get_container(&self, name: &AllotmentNamePart) -> &ContainerAllotmentStyle {
        self.root.get_container(name).unwrap_or(&self.empty_container)
    }

    pub fn get_leaf(&self, name: &AllotmentNamePart) -> &SpecifiedStyle {
        self.root.get_leaf(name).unwrap_or(&self.empty_leaf)
    }

    /* After here, used during building. Only for the use of builder. */
    pub(super) fn root(root: StyleTreeNode) -> StyleTree {
        StyleTree {
            root,
            empty_container: ContainerAllotmentStyle::empty(),
            empty_leaf: SpecifiedStyle::empty()
        }
    }
}
