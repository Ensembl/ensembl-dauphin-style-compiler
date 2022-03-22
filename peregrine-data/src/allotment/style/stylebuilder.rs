use std::{sync::{Arc, Mutex}, collections::HashMap};

use peregrine_toolkit::{lock, puzzle::{PuzzleBuilder, PuzzleValueHolder, PuzzlePiece}, log};

use crate::{allotment::{core::{arbitrator::BpPxConverter, allotmentmetadata2::AllotmentMetadata2Builder, aligner::Aligner}, boxes::{ stacker::Stacker, overlay::Overlay, bumper::Bumper }, boxes::{leaf::{FloatingLeaf}, boxtraits::Transformable, root::PlayingFieldPieces}, transformers::drawinginfo::DrawingInfo, stylespec::stylegroup::AllotmentStyleGroup}, CoordinateSystem, CoordinateSystemVariety, DataMessage};

use super::{holder::{ContainerHolder, LeafHolder}, allotmentname::{AllotmentNamePart, AllotmentName}, style::{LeafAllotmentStyle, ContainerAllotmentStyle, ContainerAllotmentType, LeafCommonStyle, LeafInheritStyle}, pendingleaf::{PendingLeaf, PendingLeafMap}};

pub struct StyleBuilder<'a> {
    aligner: Aligner,
    root: ContainerHolder,
    puzzle: PuzzleBuilder,
    converter: Arc<BpPxConverter>,
    leafs_made: HashMap<Vec<String>,LeafHolder>,
    containers_made: HashMap<Vec<String>,ContainerHolder>,
    metadata: &'a mut AllotmentMetadata2Builder,
    dustbin: FloatingLeaf
}

impl<'a> StyleBuilder<'a> {
    fn new_container(&mut self, name: &AllotmentNamePart, styles: &AllotmentStyleGroup) -> Result<(ContainerHolder,ContainerAllotmentStyle),DataMessage> {
        let style = styles.get_container(name);
        let container = match &style.allot_type {
            ContainerAllotmentType::Stack => {
                ContainerHolder::Stack(Stacker::new(&self.puzzle,&style.coord_system,&style,self.metadata,&self.aligner))
            },
            ContainerAllotmentType::Overlay => {
                ContainerHolder::Overlay(Overlay::new(&self.puzzle,&style.coord_system,&style,self.metadata,&self.aligner))
            },
            ContainerAllotmentType::Bumper => {
                ContainerHolder::Bumper(Bumper::new(&self.puzzle,&style.coord_system,&style,self.metadata,&self.aligner))
            }
        };
        Ok((container,style.clone()))
    }

    fn try_new_container(&mut self, name: &AllotmentNamePart, styles: &AllotmentStyleGroup) -> Result<ContainerHolder,DataMessage> {
        let sequence = name.sequence().to_vec();
        if let Some(container) = self.containers_made.get(&sequence) {
            Ok(container.clone())
        } else {
            let mut parent_container = if let Some((_,parent)) = name.pop() {
                if parent.empty() {
                    self.root.clone()
                } else {
                    self.try_new_container(&parent,styles)?
                }
            } else {
                self.root.clone()
            };
            let (new_container,self_conrtainer_style) = self.new_container(name,styles)?;
            parent_container.add_container(&new_container,&self_conrtainer_style)?;
            self.containers_made.insert(sequence,new_container.clone());
            Ok(new_container)
        }
    }

    fn new_floating_leaf(&self, container: &mut ContainerHolder,  name: &AllotmentNamePart, info: &DrawingInfo, styles: &AllotmentStyleGroup, leaf_style: &LeafCommonStyle) -> Result<FloatingLeaf,DataMessage> {
        let child = FloatingLeaf::new(&self.puzzle,&self.converter,&leaf_style,info,&self.aligner);
        container.add_leaf(&LeafHolder::Leaf(child.clone()),leaf_style);
        Ok(child)
    }

    fn new_leaf(&mut self, name: &AllotmentNamePart, info: &DrawingInfo, styles: &AllotmentStyleGroup, leaf_style: &LeafCommonStyle) -> Result<LeafHolder,DataMessage> {
        Ok(if let Some((_,rest)) = name.pop() {
            let mut container = self.try_new_container(&rest,styles)?;
            LeafHolder::Leaf(self.new_floating_leaf(&mut container,name,info,styles,&leaf_style)?)
        } else {
            LeafHolder::Leaf(self.dustbin.clone())
        })
    }

    fn try_new_leaf(&mut self, name: &AllotmentNamePart, info: &DrawingInfo, styles: &AllotmentStyleGroup, leaf_style: &LeafCommonStyle) -> Result<LeafHolder,DataMessage> {
        let sequence = name.sequence().to_vec();
        Ok(if let Some(leaf) = self.leafs_made.get(&sequence) {
            leaf.clone()
        } else {
            let out = self.new_leaf(name,info,styles,leaf_style)?;
            self.leafs_made.insert(sequence,out.clone());
            out
        })
    }
}

pub(crate) fn make_transformable(puzzle: &PuzzleBuilder, plm: &mut PendingLeafMap, converter: &Arc<BpPxConverter>, root: &ContainerHolder, pendings: &mut dyn Iterator<Item=&PendingLeaf>, metadata: &mut AllotmentMetadata2Builder, aligner: &Aligner) -> Result<(),DataMessage> {
    let mut styler = StyleBuilder {
        root: root.clone(),
        leafs_made: HashMap::new(),
        containers_made: HashMap::new(),
        puzzle: puzzle.clone(),
        converter: converter.clone(),
        metadata,
        dustbin: FloatingLeaf::new(puzzle,converter,&LeafCommonStyle::dustbin(),&DrawingInfo::new(),&aligner),
        aligner: aligner.clone()
    };
    for pending in pendings {
        let parts = AllotmentNamePart::new(pending.name().clone());
        let info = pending.drawing_info_clone();
        let styles = pending.style();
        let leaf_style = pending.leaf_style();
        let xformable = styler.try_new_leaf(&parts,&info,&styles,&leaf_style)?.into_tranfsormable();
        pending.set_transformable(plm,xformable);
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use std::{sync::Arc, collections::{HashMap}};

    use peregrine_toolkit::puzzle::{PuzzleBuilder, Puzzle, PuzzleSolution};

    use crate::{allotment::{core::{arbitrator::BpPxConverter, rangeused::RangeUsed, allotmentmetadata2::{AllotmentMetadata2Builder, AllotmentMetadata2}, aligner::Aligner}, boxes::root::Root, style::{allotmentname::AllotmentName, self, holder::ContainerHolder, pendingleaf::{PendingLeaf, PendingLeafMap}, stylebuilder::make_transformable}, stylespec::{stylegroup::AllotmentStyleGroup, styletreebuilder::StyleTreeBuilder, styletree::StyleTree}}};

    fn make_pendings(names: &[&str], heights: &[f64], pixel_range: &[RangeUsed<f64>], style: &AllotmentStyleGroup) -> (PendingLeafMap,Vec<PendingLeaf>) {
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
            leaf.set_style(style);
            leaf.update_drawing_info(|info| {
                info.merge_max_y(*height);
                if let Some(ref mut pixel_range) = pixel_range_iter {
                    info.merge_pixel_range(pixel_range.next().unwrap());
                }    
            });
            out.push(leaf);
        }
        (PendingLeafMap::new(),out)
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
        let mut builder = PuzzleBuilder::new();
        let converter = Arc::new(BpPxConverter::new(None));
        let root = Root::new(&mut builder);
        let aligner = Aligner::new(&root);
        let root = ContainerHolder::Root(root);
        let mut tree = StyleTreeBuilder::new();
        add_style(&mut tree, "a/", &[("padding-top","10"),("padding-bottom","5")]);
        add_style(&mut tree, "a/1", &[("depth","10"),("coordinate-system","window")]);
        let style_group = AllotmentStyleGroup::new(StyleTree::new(tree));
        let (mut plm, mut pending) = make_pendings(&["a/1","a/2","a/3","b/1","b/2","b/3"],&[1.,2.,3.],&[],&style_group);
        make_transformable(&builder,&mut plm,&converter,&root,&mut pending.iter(),&mut AllotmentMetadata2Builder::new(),&aligner);
        let puzzle = Puzzle::new(builder);
        let mut solution = PuzzleSolution::new(&puzzle);
        assert!(solution.solve());
        let transformers = pending.iter().map(|x| x.transformable(&plm).make(&solution)).collect::<Vec<_>>();
        let descs = transformers.iter().map(|x| x.describe()).collect::<Vec<_>>();
        assert_eq!(6,descs.len());
        assert!(descs[0].contains("coord_system: CoordinateSystem(Window, false)"));
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
        let mut builder = PuzzleBuilder::new();
        let converter = Arc::new(BpPxConverter::new(None));
        let root = Root::new(&mut builder);
        let aligner = Aligner::new(&root);
        let root = ContainerHolder::Root(root);
        let mut tree = StyleTreeBuilder::new();
        add_style(&mut tree, "a/", &[("padding-top","10"),("padding-bottom","5"),("type","overlay")]);        
        add_style(&mut tree, "a/1", &[("depth","10"),("coordinate-system","window")]);
        let style_group = AllotmentStyleGroup::new(StyleTree::new(tree));
        let (mut plm, mut pending) = make_pendings(&["a/1","a/2","a/3","b/1","b/2","b/3"],&[1.,2.,3.],&[],&style_group);
        make_transformable(&builder,&mut plm,&converter,&root,&mut pending.iter(),&mut AllotmentMetadata2Builder::new(),&aligner);
        let puzzle = Puzzle::new(builder);
        let mut solution = PuzzleSolution::new(&puzzle);
        assert!(solution.solve());
        let transformers = pending.iter().map(|x| x.transformable(&plm).make(&solution)).collect::<Vec<_>>();
        let descs = transformers.iter().map(|x| x.describe()).collect::<Vec<_>>();
        assert_eq!(6,descs.len());
        assert!(descs[0].contains("coord_system: CoordinateSystem(Window, false)"));
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
        let mut builder = PuzzleBuilder::new();
        let converter = Arc::new(BpPxConverter::new(None));
        let root = Root::new(&mut builder);
        let aligner = Aligner::new(&root);
        let root = ContainerHolder::Root(root);
        let ranges = [
            RangeUsed::Part(0.,3.),
            RangeUsed::Part(2.,5.),
            RangeUsed::Part(4.,7.),
            RangeUsed::Part(0.,2.),
            RangeUsed::Part(2.,4.),
            RangeUsed::Part(4.,6.)
        ];
        let mut tree = StyleTreeBuilder::new();
        add_style(&mut tree, "a/", &[("padding-top","10"),("padding-bottom","5"),("type","bumper"),("report","track")]);        
        add_style(&mut tree, "b/", &[("type","bumper"),("report","track")]);
        add_style(&mut tree, "a/1", &[("depth","10"),("coordinate-system","window")]);
        let style_group = AllotmentStyleGroup::new(StyleTree::new(tree));
        let (mut plm, pending) = make_pendings(&["a/1","a/2","a/3","b/1","b/2","b/3"],&[1.,2.,3.],&ranges,&style_group);
        let mut metadata = AllotmentMetadata2Builder::new();
        make_transformable(&builder,&mut plm,&converter,&root,&mut pending.iter(),&mut metadata,&aligner);
        let metadata = AllotmentMetadata2::new(&metadata);
        let puzzle = Puzzle::new(builder);
        let mut solution = PuzzleSolution::new(&puzzle);
        assert!(solution.solve());
        let transformers = pending.iter().map(|x| x.transformable(&plm).make(&solution)).collect::<Vec<_>>();
        let descs = transformers.iter().map(|x| x.describe()).collect::<Vec<_>>();
        assert_eq!(6,descs.len());
        println!("{:?}",descs);
        assert!(descs[0].contains("coord_system: CoordinateSystem(Window, false)"));
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
        let metadata = metadata.get(&solution);
        let metadata = metadata.summarize();
        assert_eq!(2,metadata.len());
        let (a,b) = (&metadata[0],&metadata[1]);
        assert!(a.contains_key("offset"));
        let (a,b) = if a.get("offset") == Some(&"0".to_string()) { (a,b) } else { (b,a) };
        assert_eq!(Some(&"track".to_string()),a.get("type"));
        assert_eq!(Some(&"0".to_string()),a.get("offset"));
        assert_eq!(Some(&"21".to_string()),a.get("height"));
        assert_eq!(Some(&"track".to_string()),b.get("type"));
        assert_eq!(Some(&"21".to_string()),b.get("offset"));
        assert_eq!(Some(&"24".to_string()),b.get("height"));
    }
}
