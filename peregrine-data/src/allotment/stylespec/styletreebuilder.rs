use std::{collections::HashMap};

use crate::allotment::style::{allotmentname::{AllotmentName, AllotmentNamePart}, style::{LeafAllotmentStyle, ContainerAllotmentStyle}};

use super::styletree::{StyleTree, StyleTreeNode};

const TOKEN_ANY : &str = "*";

/* NB: In StyleTreeNodes, None means "other", ie all * properties which are not overridden are also propagated to extant
 * not-None leaves.
 */

#[derive(Debug)]
struct BuilderNode {
    container: HashMap<String,String>,
    leaf: HashMap<String,String>,
    children: HashMap<Option<String>,BuilderNode>,
    all: bool
}

impl BuilderNode {
    fn new() -> BuilderNode {
        BuilderNode {
            container: HashMap::new(),
            leaf: HashMap::new(),
            children: HashMap::new(),
            all: false
        }
    }
}

impl BuilderNode {
    fn get_node(&mut self, name: &AllotmentNamePart) -> &mut BuilderNode {
        if let Some((head,tail)) = name.shift() {
            let parent = self.get_node(&tail);
            let name = if head == TOKEN_ANY { None } else { Some(head) };
            parent.children.entry(name).or_insert_with(|| BuilderNode::new())
        } else {
            self
        }
    }

    fn add(&mut self, mut properties: HashMap<String,String>, container: bool) {
        let map = if container { &mut self.container } else { &mut self.leaf };
        for (key,value) in properties.drain() {
            map.insert(key,value);
        }
    }

    fn merge_self(&mut self, other: &BuilderNode) {
        for (k,v) in other.container.iter() {
            if !self.container.contains_key(k) {
                self.container.insert(k.to_string(),v.to_string());
            }
        }
        for (k,v) in other.leaf.iter() {
            if !self.leaf.contains_key(k) {
                self.leaf.insert(k.to_string(),v.to_string());
            }
        }
    }

    fn merge_all(&mut self, all: &BuilderNode) {
        for (name,all_node) in all.children.iter() {
            if !self.children.contains_key(name) {
                self.children.insert(name.clone(),BuilderNode::new());
            }
            self.children.get_mut(name).unwrap().merge_all(all_node);
        }
        self.merge_self(all);
    }

    fn add_any(&mut self) {
        for child in self.children.values_mut() {
            child.add_any();
        }
        if let Some(all) = self.children.remove(&None) {
            for child in self.children.values_mut() {
                child.merge_all(&all);
            }
            self.children.insert(None,all);
        }
    }

    fn add_all(&mut self, node: &BuilderNode) {
        for child in self.children.values_mut() {
            child.add_all(node);
        }
        self.merge_self(node);
        if !self.children.contains_key(&None) {
            self.all = true;
            let mut child = BuilderNode::new();
            child.merge_self(node);
            self.children.insert(None,child);
        }
    }

    fn build(&self) -> StyleTreeNode {
        let container = ContainerAllotmentStyle::build(&self.container);
        let leaf = LeafAllotmentStyle::build(&self.leaf);
        let mut node = StyleTreeNode::new(container,leaf,self.all);
        for (name,child) in &self.children {
            node.add(name.as_ref(),child.build());
        }
        node
    }
}

#[derive(Debug)]
pub struct StyleTreeBuilder {
    root: BuilderNode,
    all: Vec<(AllotmentNamePart,BuilderNode)> // not a proper tree node: never has children
}

impl StyleTreeBuilder {
    pub fn new() -> StyleTreeBuilder {
        StyleTreeBuilder {
            root: BuilderNode::new(),
            all: vec![]
        }
    }

    pub fn add(&mut self, spec: &str, properties: HashMap<String,String>) {
        let mut name = AllotmentNamePart::new(AllotmentName::new(spec));
        let container = name.full().is_container();
        if name.remove_all() {
            let mut node = BuilderNode::new();
            node.add(properties,container);
            self.all.push((name,node));
        } else {
            let node = self.root.get_node(&name);
            node.add(properties,container);
        }
    }

    pub(super) fn build(&mut self) -> StyleTree {
        println!("{:?}",self);
        for (name,node) in &self.all {
            self.root.get_node(name).add_all(node);
        }
        self.all = vec![];
        self.root.add_any();
        StyleTree::root(self.root.build()) 
    }
}

#[cfg(test)]
mod test {
    use crate::allotment::{style::allotmentname::AllotmentName };

    use super::*;

    fn add(builder: &mut StyleTreeBuilder, spec: &str, props: &[(&str,&str)]) {
        let mut properties = HashMap::new();
        for (k,v) in props {
            properties.insert(k.to_string(),v.to_string());
        }
        builder.add(spec,properties);
    }

    fn container(tree: &StyleTree, spec: &str) -> (f64,f64) {
        let container = tree.get_container(&AllotmentNamePart::new(AllotmentName::new(spec)));
        (container.padding.padding_top,container.padding.padding_bottom)
    }

    fn leaf(tree: &StyleTree, spec: &str) -> i8 {
        let leaf = tree.get_leaf(&AllotmentNamePart::new(AllotmentName::new(spec)));
        leaf.leaf.make(leaf).depth
    }

    #[test]
    fn styletree_smoke() {
        let mut builder = StyleTreeBuilder::new();
        add(&mut builder,"x", &[("depth","1")]);
        add(&mut builder,"y", &[("depth","2")]);
        add(&mut builder,"**/z", &[("depth","3")]);
        add(&mut builder,"a/", &[("padding-top","1")]);
        add(&mut builder,"b/", &[("padding-top","2")]);
        add(&mut builder,"c/", &[("padding-top","3")]);
        add(&mut builder,"a/m/", &[("padding-bottom","2")]);
        add(&mut builder,"*/n/", &[("padding-bottom","3")]);
        add(&mut builder,"*/n/*", &[("depth","4")]);
        add(&mut builder,"*/*/j", &[("depth","5")]);
        add(&mut builder,"**/k", &[("depth","6")]);
        add(&mut builder,"**/*/m", &[("depth","7")]);
        let tree = StyleTree::new(builder);
        assert_eq!(1,leaf(&tree,"x"));
        assert_eq!(2,leaf(&tree,"y"));
        assert_eq!(3,leaf(&tree,"z"));
        assert_eq!((1.,0.),container(&tree,"a/"));
        assert_eq!((0.,2.),container(&tree,"a/m/"));
        assert_eq!((0.,3.),container(&tree,"a/n/"));
        assert_eq!((0.,0.),container(&tree,"a/p/"));
        assert_eq!((0.,3.),container(&tree,"x/n/"));
        assert_eq!(0,leaf(&tree,"w"));
        assert_eq!(4,leaf(&tree,"x/n/w"));
        assert_eq!(3,leaf(&tree,"a/z"));
        assert_eq!(3,leaf(&tree,"x/z"));
        assert_eq!(4,leaf(&tree,"x/n/z"));
        assert_eq!(0,leaf(&tree,"j"));
        assert_eq!(0,leaf(&tree,"t/j"));
        assert_eq!(5,leaf(&tree,"t/t/j"));
        assert_eq!(0,leaf(&tree,"t/t/t/j"));
        assert_eq!(6,leaf(&tree,"k"));
        assert_eq!(6,leaf(&tree,"t/k"));
        assert_eq!(6,leaf(&tree,"t/t/k"));
        assert_eq!(6,leaf(&tree,"t/t/t/k"));
    }
}
