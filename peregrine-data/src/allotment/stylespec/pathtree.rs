/* We choose themost flexible andsimple algorithm, hoping that thecost isn't prohibitive given
 * the limited data-set size andcomplexity.
 */

use std::collections::HashMap;
use std::hash::Hash;
use std::fmt;

#[derive(PartialEq,Eq,Hash,Clone)]
pub(crate) enum PathKey<K> where K: PartialEq+Eq+Hash+Clone {
    Fixed(K),
    AnyOne,
    AnyMany
}

struct PathTreeNode<K,V> where K: PartialEq+Eq+Hash+Clone {
    prefix_properties: Option<V>,
    final_properties: Option<V>,
    fixed_children: HashMap<K,Box<PathTreeNode<K,V>>>,
    one_kid: Option<Box<PathTreeNode<K,V>>>,
    many_kid: Option<Box<PathTreeNode<K,V>>>
}

impl<K,V> PathTreeNode<K,V> where K: PartialEq+Eq+Hash+Clone {
    fn empty() -> PathTreeNode<K,V> {
        PathTreeNode {
            prefix_properties: None,
            final_properties: None,
            fixed_children: HashMap::new(),
            one_kid: None,
            many_kid: None
        }
    }

    /* NOTE: path is reversed! */
    fn add<F,M>(&mut self, merge: F, mut path: Vec<PathKey<K>>, value: M) where F: Fn(&mut Option<V>,M) {
        if let Some(first) = path.pop() {
            if path.len() == 1 {
                if let PathKey::AnyMany = path[0] {
                    merge(&mut self.prefix_properties,value);
                    return;
                }
            }
            let kid = match first {
                PathKey::Fixed(k) => self.fixed_children.entry(k).or_insert_with(|| Box::new(PathTreeNode::empty())),
                PathKey::AnyOne => self.one_kid.get_or_insert_with(|| Box::new(PathTreeNode::empty())),
                PathKey::AnyMany => self.many_kid.get_or_insert_with(|| Box::new(PathTreeNode::empty())),
            };
            kid.add(merge,path,value);
        } else {
            merge(&mut self.final_properties,value);
        }
    }

    fn lookup<F>(&self, merge: &F, output: &mut Option<V>, input: &[K]) where F: Fn(&mut Option<V>,&V) {
        if input.len() == 0 {
            if let Some(properties) = &self.final_properties {
                merge(output,properties);
            }
        } else {
            if let Some(kid) = self.fixed_children.get(&input[0]) {
                kid.lookup(merge,output,&input[1..]);
            }
            if let Some(kid) = &self.one_kid {
                kid.lookup(merge,output,&input[1..]);
            }
        }
        if let Some(kid) = &self.many_kid {
            for subpath in 0..input.len() {
                kid.lookup(merge,output,&input[subpath..]);
            }
        }
        if let Some(properties) = &self.prefix_properties {
            merge(output,properties);
        }
    }

    fn lookup_suffixes<F>(&self, merge: &F, output: &mut Option<V>, input: &[K]) where F: Fn(&mut Option<V>,&V) {
        for chop in 0..input.len() {
            self.lookup(merge,output,&input[chop..]);
        }
    }
}

pub(super) struct PathTree<M,K,V> where K: PartialEq+Eq+Hash+Clone {
    merge_add: Box<dyn Fn(&mut Option<V>,M)>,
    merge_lookup: Box<dyn Fn(&mut Option<V>,&V)>,
    tree: PathTreeNode<K,V>
}

impl<M,K,V> PathTree<M,K,V> where K: PartialEq+Eq+Hash+Clone {
    pub(super) fn new<F,G>(merge_add: F, merge_lookup: G) -> PathTree<M,K,V> 
            where F: Fn(&mut Option<V>,M) + 'static, G: Fn(&mut Option<V>,&V) + 'static {
        PathTree {
            merge_add: Box::new(merge_add),
            merge_lookup: Box::new(merge_lookup),
            tree: PathTreeNode::empty()
        }
    }

    pub(super) fn add(&mut self, path: &[PathKey<K>], value: M) {
        let mut path = path.to_vec();
        path.reverse();
        self.tree.add(&self.merge_add,path,value);
    }

    pub(super) fn lookup(&self, path: &[K]) -> Option<V> {
        let mut out = None;
        self.tree.lookup(&self.merge_lookup,&mut out,path);
        out
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_pathtree_smoke() {
        let mut root = PathTree::new(
            |all,new| { all.get_or_insert(vec![]).push(new); },
            |output,input| { output.get_or_insert_with(|| vec![]).extend(input.iter().cloned()); });
        root.add(&[PathKey::Fixed("x")],1);
        root.add(&[PathKey::Fixed("y")],2);
        root.add(&[PathKey::AnyMany,PathKey::Fixed("z")],3);
        root.add(&[PathKey::Fixed("a")],4);
        root.add(&[PathKey::Fixed("b")],5);
        root.add(&[PathKey::Fixed("c")],6);
        root.add(&[PathKey::Fixed("a"),PathKey::Fixed("m")],7);
        root.add(&[PathKey::AnyOne,PathKey::Fixed("n")],8);
        root.add(&[PathKey::AnyOne,PathKey::Fixed("n"),PathKey::AnyOne],9);
        root.add(&[PathKey::AnyOne,PathKey::AnyOne,PathKey::Fixed("j")],10);
        root.add(&[PathKey::AnyMany,PathKey::Fixed("k")],11);
        root.add(&[PathKey::AnyMany,PathKey::AnyOne,PathKey::Fixed("m")],12);
        /**/
        assert_eq!(Some(vec![1]),root.lookup(&["x"]));
        assert_eq!(Some(vec![2]),root.lookup(&["y"]));
        assert_eq!(Some(vec![3]),root.lookup(&["z"]));
        assert_eq!(Some(vec![4]),root.lookup(&["a"]));
        assert_eq!(Some(vec![7,12]),root.lookup(&["a","m"]));
        assert_eq!(Some(vec![8]),root.lookup(&["a","n"]));
        assert!(root.lookup(&["a","p"]).is_none());
        assert_eq!(Some(vec![8]),root.lookup(&["x","n"]));
        assert!(root.lookup(&["w"]).is_none());
        assert_eq!(Some(vec![9]),root.lookup(&["x","n","w"]));
        assert_eq!(Some(vec![3]),root.lookup(&["a","z"]));
        assert_eq!(Some(vec![3]),root.lookup(&["x","z"]));
        assert_eq!(Some(vec![9,3]),root.lookup(&["x","n","z"]));
        assert!(root.lookup(&["j"]).is_none());
        assert!(root.lookup(&["t","j"]).is_none());
        assert_eq!(Some(vec![10]),root.lookup(&["t","t","j"]));
        assert!(root.lookup(&["t","t","t","j"]).is_none());
        assert_eq!(Some(vec![11]),root.lookup(&["k"]));
        assert_eq!(Some(vec![11]),root.lookup(&["t","k"]));
        assert_eq!(Some(vec![11]),root.lookup(&["t","t","k"]));
        assert_eq!(Some(vec![11]),root.lookup(&["t","t","t","k"]));        
    }

    #[test]
    fn styletree_bug() {
        let mut root = PathTree::new(
            |all,new| { *all = Some(new); },
            |output,input| { output.get_or_insert_with(|| vec![]).extend(input.iter().cloned()); }
        );
        root.add(&[PathKey::AnyMany,PathKey::Fixed("a"),PathKey::AnyOne],vec![1]);
        root.add(&[PathKey::Fixed("b"),PathKey::Fixed("c")],vec![2]);
        assert_eq!(Some(vec![1]),root.lookup(&["b","c","a","c"]));
    }

}
