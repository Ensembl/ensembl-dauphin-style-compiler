use std::{sync::{Arc, Mutex}, collections::HashMap};

use peregrine_toolkit::{lock, puzzle::{PuzzleBuilder, PuzzleValueHolder, PuzzlePiece}};

use crate::{allotment::{core::arbitrator::BpPxConverter, boxes::{ stacker::Stacker, overlay::Overlay, bumper::Bumper }, boxes::{leaf::{FloatingLeaf}, boxtraits::Transformable}, transformers::drawinginfo::DrawingInfo, stylespec::stylegroup::AllotmentStyleGroup}, CoordinateSystem, CoordinateSystemVariety};

use super::{holder::{ContainerHolder, LeafHolder}, allotmentname::{AllotmentNamePart, AllotmentName}, style::{LeafAllotmentStyle, ContainerAllotmentStyle, ContainerAllotmentType, LeafCommonStyle}, pendingleaf::PendingLeaf};

pub struct StyleBuilder {
    root: ContainerHolder,
    puzzle: PuzzleBuilder,
    converter: Arc<BpPxConverter>,
    leafs_made: HashMap<Vec<String>,LeafHolder>,
    containers_made: HashMap<Vec<String>,ContainerHolder>,
    anchors: HashMap<String,PuzzlePiece<f64>>,
    dustbin: FloatingLeaf
}

impl StyleBuilder {
    fn new_container(&self, name: &AllotmentNamePart, styles: &AllotmentStyleGroup) -> ContainerHolder {
        let style = styles.get_container(name);
        match style.allot_type {
            ContainerAllotmentType::Stack => {
                ContainerHolder::Stack(Stacker::new(&self.puzzle,&style.padding))
            },
            ContainerAllotmentType::Overlay => {
                ContainerHolder::Overlay(Overlay::new(&self.puzzle,&style.padding))
            },
            ContainerAllotmentType::Bumper => {
                ContainerHolder::Bumper(Bumper::new(&self.puzzle,&style.padding))
            }
        }
    }

    fn try_new_container(&mut self, name: &AllotmentNamePart, styles: &AllotmentStyleGroup) -> ContainerHolder {
        let sequence = name.sequence().to_vec();
        if let Some(container) = self.containers_made.get(&sequence) {
            container.clone()
        } else {
            let mut parent = if let Some((_,parent)) = name.pop() {
                self.try_new_container(&parent,styles)
            } else {
                self.root.clone()
            };
            let out = self.new_container(name,styles);
            parent.add_container(&out);
            self.containers_made.insert(sequence,out.clone());
            out
        }
    }

    fn new_floating_leaf(&self, container: &mut ContainerHolder,  name: &AllotmentNamePart, info: &DrawingInfo, styles: &AllotmentStyleGroup) -> FloatingLeaf {
        let style = styles.get_leaf(name);
        let child = FloatingLeaf::new(&self.puzzle,&self.converter,&style.leaf,info);
        container.add_leaf(&LeafHolder::Leaf(child.clone()));
        child
    }

    fn new_leaf(&mut self, name: &AllotmentNamePart, info: &DrawingInfo, styles: &AllotmentStyleGroup) -> LeafHolder {
        if let Some((_,rest)) = name.pop() {
            let mut container = self.try_new_container(&rest,styles);
            LeafHolder::Leaf(self.new_floating_leaf(&mut container,name,info,styles))
        } else {
            LeafHolder::Leaf(self.dustbin.clone())
        }
    }

    fn try_new_leaf(&mut self, name: &AllotmentNamePart, info: &DrawingInfo, styles: &AllotmentStyleGroup) -> LeafHolder {
        let sequence = name.sequence().to_vec();
        if let Some(leaf) = self.leafs_made.get(&sequence) {
            leaf.clone()
        } else {
            let out = self.new_leaf(name,info,styles);
            self.leafs_made.insert(sequence,out.clone());
            out
        }
    }
}

pub(crate) fn make_transformable(puzzle: &PuzzleBuilder, converter: &Arc<BpPxConverter>, root: &ContainerHolder, pendings: &mut dyn Iterator<Item=&PendingLeaf>, styles: &AllotmentStyleGroup) {
    let mut styler = StyleBuilder {
        root: root.clone(),
        leafs_made: HashMap::new(),
        containers_made: HashMap::new(),
        anchors: HashMap::new(),
        puzzle: puzzle.clone(),
        converter: converter.clone(),
        dustbin: FloatingLeaf::new(puzzle,converter,&LeafCommonStyle::dustbin(),&DrawingInfo::new())
    };
    for pending in pendings {
        let parts = AllotmentNamePart::new(pending.name().clone());
        let info = pending.drawing_info_clone();
        let xformable = styler.try_new_leaf(&parts,&info,styles).into_tranfsormable();
        pending.set_transformable(xformable);
    }
}

#[cfg(test)]
mod test {
    use std::{sync::Arc, collections::{HashMap, btree_map::Range}};

    use peregrine_toolkit::puzzle::{PuzzleBuilder, Puzzle, PuzzleSolution};

    use crate::{allotment::{core::{arbitrator::BpPxConverter, rangeused::RangeUsed}, boxes::root::Root, style::{allotmentname::AllotmentName, self, holder::ContainerHolder, pendingleaf::PendingLeaf, stylebuilder::make_transformable}, stylespec::{stylegroup::AllotmentStyleGroup, styletreebuilder::StyleTreeBuilder, styletree::StyleTree}}};

    fn make_pendings(names: &[&str], heights: &[f64], pixel_range: &[RangeUsed<f64>]) -> Vec<PendingLeaf> {
        let heights = if heights.len() > 0 {
            heights.iter().cycle()
        } else {
            [0.].iter().cycle()
        };
        let mut pixel_range_iter = if pixel_range.len() > 0 {
            Some(pixel_range.iter().cycle())
        } else {
            None
        };
        let mut out = vec![];
        for (name,height) in names.iter().zip(heights) {
            let mut leaf = PendingLeaf::new(&AllotmentName::new(name));
            leaf.update_drawing_info(|info| {
                info.merge_max_y(*height);
                if let Some(ref mut pixel_range) = pixel_range_iter {
                    info.merge_pixel_range(pixel_range.next().unwrap());
                }    
            });
            out.push(leaf);
        }
        out
    }

    fn add_style(group: &mut StyleTreeBuilder, name: &str, values: &[(&str,&str)]) {
        let mut values_hash = HashMap::new();
        for (k,v) in values {
            values_hash.insert(k.to_string(),v.to_string());
        }
        group.add(name,values_hash);
    }

    // XXX generic specs
    // XXX errors

    #[test]
    fn allotment_smoke() {
        let builder = PuzzleBuilder::new();
        let converter = Arc::new(BpPxConverter::new(None));
        let root = ContainerHolder::Root(Root::new());
        let mut pending = make_pendings(&["a/1","a/2","a/3","b/1","b/2","b/3"],&[1.,2.,3.],&[]);
        let mut tree = StyleTreeBuilder::new();
        add_style(&mut tree, "a/", &[("padding-top","10"),("padding-bottom","5")]);        
        add_style(&mut tree, "a/1", &[("depth","10"),("coordinate-system","window")]);
        let style_group = AllotmentStyleGroup::new(StyleTree::new(tree));
        make_transformable(&builder,&converter,&root,&mut pending.iter(),&style_group);
        let puzzle = Puzzle::new(builder);
        let mut solution = PuzzleSolution::new(&puzzle);
        assert!(solution.solve());
        let transformers = pending.iter().map(|x| x.transformable().make(&solution)).collect::<Vec<_>>();
        let descs = transformers.iter().map(|x| x.describe()).collect::<Vec<_>>();
        assert_eq!(6,descs.len());
        assert!(descs[0].contains("dustbin: false"));
        assert!(descs[0].contains("coord_system: CoordinateSystem(Tracking, false)"));
        assert!(descs[0].contains("top: 10.0"));
        assert!(descs[0].contains("height: 1.0"));
        assert!(descs[1].contains("top: 11.0"));
        assert!(descs[1].contains("height: 2.0"));
        assert!(descs[2].contains("top: 13.0"));
        assert!(descs[2].contains("height: 3.0"));
        assert!(descs[3].contains("top: 21.0"));
        assert!(descs[3].contains("height: 1.0"));
        assert!(descs[4].contains("top: 22.0"));
        assert!(descs[4].contains("height: 2.0"));
        assert!(descs[5].contains("top: 24.0"));
        assert!(descs[5].contains("height: 3.0"));
    }

    #[test]
    fn allotment_overlay() {
        let builder = PuzzleBuilder::new();
        let converter = Arc::new(BpPxConverter::new(None));
        let root = ContainerHolder::Root(Root::new());
        let mut pending = make_pendings(&["a/1","a/2","a/3","b/1","b/2","b/3"],&[1.,2.,3.],&[]);
        let mut tree = StyleTreeBuilder::new();
        add_style(&mut tree, "a/", &[("padding-top","10"),("padding-bottom","5"),("type","overlay")]);        
        add_style(&mut tree, "a/1", &[("depth","10"),("coordinate-system","window")]);
        let style_group = AllotmentStyleGroup::new(StyleTree::new(tree));
        make_transformable(&builder,&converter,&root,&mut pending.iter(),&style_group);
        let puzzle = Puzzle::new(builder);
        let mut solution = PuzzleSolution::new(&puzzle);
        assert!(solution.solve());
        let transformers = pending.iter().map(|x| x.transformable().make(&solution)).collect::<Vec<_>>();
        let descs = transformers.iter().map(|x| x.describe()).collect::<Vec<_>>();
        assert_eq!(6,descs.len());
        assert!(descs[0].contains("dustbin: false"));
        assert!(descs[0].contains("coord_system: CoordinateSystem(Tracking, false)"));
        assert!(descs[0].contains("top: 10.0"));
        assert!(descs[0].contains("height: 1.0"));
        assert!(descs[1].contains("top: 10.0"));
        assert!(descs[1].contains("height: 2.0"));
        assert!(descs[2].contains("top: 10.0"));
        assert!(descs[2].contains("height: 3.0"));
        assert!(descs[3].contains("top: 18.0"));
        assert!(descs[3].contains("height: 1.0"));
        assert!(descs[4].contains("top: 19.0"));
        assert!(descs[4].contains("height: 2.0"));
        assert!(descs[5].contains("top: 21.0"));
        assert!(descs[5].contains("height: 3.0"));
    }

    #[test]
    fn allotment_bumper() {
        let builder = PuzzleBuilder::new();
        let converter = Arc::new(BpPxConverter::new(None));
        let root = ContainerHolder::Root(Root::new());
        let ranges = [
            RangeUsed::Part(0.,3.),
            RangeUsed::Part(2.,5.),
            RangeUsed::Part(4.,7.),
            RangeUsed::Part(0.,2.),
            RangeUsed::Part(2.,4.),
            RangeUsed::Part(4.,6.)
        ];
        let mut pending = make_pendings(&["a/1","a/2","a/3","b/1","b/2","b/3"],&[1.,2.,3.],&ranges);
        let mut tree = StyleTreeBuilder::new();
        add_style(&mut tree, "a/", &[("padding-top","10"),("padding-bottom","5"),("type","bumper")]);        
        add_style(&mut tree, "b/", &[("type","bumper")]);
        add_style(&mut tree, "a/1", &[("depth","10"),("coordinate-system","window")]);
        let style_group = AllotmentStyleGroup::new(StyleTree::new(tree));
        make_transformable(&builder,&converter,&root,&mut pending.iter(),&style_group);
        let puzzle = Puzzle::new(builder);
        let mut solution = PuzzleSolution::new(&puzzle);
        assert!(solution.solve());
        let transformers = pending.iter().map(|x| x.transformable().make(&solution)).collect::<Vec<_>>();
        let descs = transformers.iter().map(|x| x.describe()).collect::<Vec<_>>();
        assert_eq!(6,descs.len());
        println!("{:?}",descs);
        assert!(descs[0].contains("dustbin: false"));
        assert!(descs[0].contains("coord_system: CoordinateSystem(Tracking, false)"));
        assert!(descs[0].contains("top: 15.0"));
        assert!(descs[0].contains("height: 1.0"));
        assert!(descs[1].contains("top: 13.0"));
        assert!(descs[1].contains("height: 2.0"));
        assert!(descs[2].contains("top: 10.0"));
        assert!(descs[2].contains("height: 3.0"));
        assert!(descs[3].contains("top: 21.0"));
        assert!(descs[3].contains("height: 1.0"));
        assert!(descs[4].contains("top: 21.0"));
        assert!(descs[4].contains("height: 2.0"));
        assert!(descs[5].contains("top: 21.0"));
        assert!(descs[5].contains("height: 3.0"));
    }
}
