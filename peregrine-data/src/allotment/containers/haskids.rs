use std::{collections::HashMap, sync::Arc};
use crate::{allotment::{core::{allotmentname::AllotmentName}, leafs::floating::FloatingLeaf, style::{styletree::StyleTree, containerstyle::{ContainerAllotmentType, ContainerStyle}}, layout::layouttree::ContainerOrLeaf}, LeafRequest};
use super::{container::{Container, ChildKeys, ContainerSpecifics}, bumper::Bumper, stacker::Stacker, overlay::Overlay};

fn new_container(name: &AllotmentName, style: &ContainerStyle) -> Box<dyn ContainerSpecifics + 'static> {
    match &style.allot_type {
        ContainerAllotmentType::Stack => Box::new(Stacker::new()),
        ContainerAllotmentType::Overlay => Box::new(Overlay::new()),
        ContainerAllotmentType::Bumper => Box::new(Bumper::new(name,false)),
        ContainerAllotmentType::Wall => Box::new(Bumper::new(name,true)),
    }
}

fn new_leaf(pending: &LeafRequest, name: &AllotmentName) -> FloatingLeaf {
    let drawing_info = pending.shape_bounds(|di| di.clone());
    let child = FloatingLeaf::new(name,&pending.leaf_style(),&drawing_info);
    child
}

pub(super) struct HasKids {
    pub(super) children: HashMap<ChildKeys,Box<dyn ContainerOrLeaf>>,
    child_leafs: HashMap<String,FloatingLeaf>,
}

impl HasKids {
    pub(super) fn new() -> HasKids {
        HasKids {
            children: HashMap::new(),
            child_leafs: HashMap::new(),   
        }
    }

    pub(super) fn get_leaf(&mut self, pending: &LeafRequest, cursor: usize, styles: &Arc<StyleTree>) -> FloatingLeaf {
        let name = pending.name().sequence();
        let step = &name[cursor];
        if cursor == name.len() - 1 {
            /* leaf */
            if !self.child_leafs.contains_key(step) {
                /* create leaf */
                let name = name[0..(cursor+1)].iter().map(|x| x.to_string()).collect::<Vec<_>>();
                let name = AllotmentName::do_new(name);
                let leaf = new_leaf(pending,&name);
                self.child_leafs.insert(step.to_string(),leaf.clone());
                self.children.insert(ChildKeys::Leaf(step.to_string()),Box::new(leaf.clone()));
            }
            self.child_leafs.get(step).unwrap().clone()
        } else {
            /* container */
            let key = ChildKeys::Container(step.to_string());
            if !self.children.contains_key(&key) {
                /* create container */
                let name = name[0..(cursor+1)].iter().map(|x| x.to_string()).collect::<Vec<_>>();
                let name = AllotmentName::do_new(name);
                let style = styles.lookup_container(&name);
                let container = Container::new(&name,&style,new_container(&name,&style));
                self.children.insert(key.clone(),Box::new(container));
            }
            self.children.get_mut(&key).unwrap().get_leaf(pending,cursor+1,styles).clone()
        }
    }
}
