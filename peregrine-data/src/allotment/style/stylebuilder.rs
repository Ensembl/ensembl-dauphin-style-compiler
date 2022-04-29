use std::{collections::HashMap};
use crate::{allotment::{core::{carriageoutput::BoxPositionContext, trainstate::CarriageTrainStateSpec}, boxes::{ stacker::Stacker, overlay::Overlay, bumper::Bumper }, boxes::{leaf::{FloatingLeaf}}, transformers::drawinginfo::DrawingInfo, stylespec::stylegroup::AllotmentStyleGroup}, DataMessage, LeafRequest};
use super::{holder::{ContainerHolder, LeafHolder}, allotmentname::{AllotmentNamePart, AllotmentName}, style::{ContainerAllotmentType, LeafCommonStyle}};

struct StyleBuilder<'a> {
    root: ContainerHolder,
    leafs_made: HashMap<Vec<String>,LeafHolder>,
    containers_made: HashMap<Vec<String>,ContainerHolder>,
    prep: &'a mut BoxPositionContext,
    dustbin: FloatingLeaf
}

impl<'a> StyleBuilder<'a> {
    fn new(prep: &'a mut BoxPositionContext) -> StyleBuilder<'a> {
        let dustbin_name = AllotmentNamePart::new(AllotmentName::new(""));
        StyleBuilder {
            root: ContainerHolder::Root(prep.root.clone()),
            leafs_made: HashMap::new(),
            containers_made: HashMap::new(),
            dustbin: FloatingLeaf::new(&dustbin_name,&prep.bp_px_converter,&LeafCommonStyle::dustbin(),&DrawingInfo::new()),
            prep
        }
    }

    fn new_container(&mut self, name: &AllotmentNamePart, styles: &AllotmentStyleGroup) -> Result<ContainerHolder,DataMessage> {
        let style = styles.get_container(name);
        let container = match &style.allot_type {
            ContainerAllotmentType::Stack => {
                ContainerHolder::Stack(Stacker::new(name,&style))
            },
            ContainerAllotmentType::Overlay => {
                ContainerHolder::Overlay(Overlay::new(name,&style))
            },
            ContainerAllotmentType::Bumper => {
                ContainerHolder::Bumper(Bumper::new(name,&style))
            }
        };
        Ok(container)
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
            let new_container = self.new_container(name,styles)?;
            parent_container.add_container(&new_container)?;
            self.containers_made.insert(sequence,new_container.clone());
            Ok(new_container)
        }
    }

    fn new_floating_leaf(&self, pending: &LeafRequest, name: &AllotmentNamePart, container: &mut ContainerHolder) -> Result<FloatingLeaf,DataMessage> {
        let child = FloatingLeaf::new(name,&self.prep.bp_px_converter,&pending.leaf_style(),&pending.drawing_info_clone());
        container.add_leaf(&LeafHolder::Leaf(child.clone()));
        Ok(child)
    }

    fn new_leaf(&mut self, pending: &LeafRequest, name: &AllotmentNamePart) -> Result<LeafHolder,DataMessage> {
        Ok(if let Some((_,rest)) = name.pop() {
            let mut container = self.try_new_container(&rest,&pending.style())?;
            LeafHolder::Leaf(self.new_floating_leaf(pending,name,&mut container)?)
        } else {
            LeafHolder::Leaf(self.dustbin.clone())
        })
    }

    fn try_new_leaf(&mut self, pending: &LeafRequest) -> Result<LeafHolder,DataMessage> {
        let name = AllotmentNamePart::new(pending.name().clone());
        let sequence = name.sequence().to_vec();
        Ok(if let Some(leaf) = self.leafs_made.get(&sequence) {
            leaf.clone()
        } else {
            let out = self.new_leaf(pending,&name)?;
            self.leafs_made.insert(sequence,out.clone());
            out
        })
    }
}

pub(crate) fn make_transformable(prep: &mut BoxPositionContext, pendings: &mut dyn Iterator<Item=&LeafRequest>) -> Result<CarriageTrainStateSpec,DataMessage> {
    /* Build box tree */
    let mut styler = StyleBuilder::new(prep);
    for pending in pendings {
        let xformable = styler.try_new_leaf(&pending)?.into_tranfsormable();
        styler.prep.plm.set_transformable(&pending.name(),&xformable);
    }
    /* Wire box tree */
    let state_spec = prep.root.clone().build(prep);
    Ok(state_spec)
}

#[cfg(test)]
mod test {
    use std::{sync::{Arc}, collections::{HashMap}};

    use peregrine_toolkit::{puzzle::{AnswerAllocator}};

    use crate::{allotment::{core::{carriageoutput::BoxPositionContext}, style::{allotmentname::AllotmentName, stylebuilder::make_transformable}, stylespec::{stylegroup::AllotmentStyleGroup, styletreebuilder::StyleTreeBuilder, styletree::StyleTree}, util::{bppxconverter::BpPxConverter, rangeused::RangeUsed}, globals::allotmentmetadata::{LocalAllotmentMetadata, GlobalAllotmentMetadataBuilder, GlobalAllotmentMetadata}}, LeafRequest};

    fn make_pendings(names: &[&str], heights: &[f64], pixel_range: &[RangeUsed<f64>], style: &AllotmentStyleGroup) -> Vec<LeafRequest> {
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
            let leaf = LeafRequest::new(&AllotmentName::new(name));
            leaf.set_style(style);
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
        let mut tree = StyleTreeBuilder::new();
        add_style(&mut tree, "z/a/", &[("padding-top","10"),("padding-bottom","5")]);
        add_style(&mut tree, "z/a/1", &[("depth","10"),("coordinate-system","window")]);
        let style_group = AllotmentStyleGroup::new(StyleTree::new(tree));
        let pending = make_pendings(&["z/a/1","z/a/2","z/a/3","z/b/1","z/b/2","z/b/3"],&[1.,2.,3.],&[],&style_group);
        let mut prep = BoxPositionContext::new(None);
        assert!(make_transformable(&mut prep,&mut pending.iter()).ok().is_some());
        let mut aia = AnswerAllocator::new();
        let answer_index = aia.get();
        let transformers = pending.iter().map(|x| prep.plm.transformable(x.name()).make(&answer_index)).collect::<Vec<_>>();
        let descs = transformers.iter().map(|x| x.describe()).collect::<Vec<_>>();
        println!("{:?}",descs);
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
        let mut tree = StyleTreeBuilder::new();
        add_style(&mut tree, "z/a/", &[("padding-top","10"),("padding-bottom","5"),("type","overlay")]);        
        add_style(&mut tree, "z/a/1", &[("depth","10"),("coordinate-system","window")]);
        let style_group = AllotmentStyleGroup::new(StyleTree::new(tree));
        let pending = make_pendings(&["z/a/1","z/a/2","z/a/3","z/b/1","z/b/2","z/b/3"],&[1.,2.,3.],&[],&style_group);
        let mut prep = BoxPositionContext::new(None);
        assert!(make_transformable(&mut prep,&mut pending.iter()).ok().is_some());
        let mut aia = AnswerAllocator::new();
        let answer_index = aia.get();
        let transformers = pending.iter().map(|x| prep.plm.transformable(x.name()).make(&answer_index)).collect::<Vec<_>>();
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
        let ranges = [
            RangeUsed::Part(0.,3.),
            RangeUsed::Part(2.,5.),
            RangeUsed::Part(4.,7.),
            RangeUsed::Part(0.,2.),
            RangeUsed::Part(2.,4.),
            RangeUsed::Part(4.,6.)
        ];
        let mut tree = StyleTreeBuilder::new();
        add_style(&mut tree, "z/a/", &[("padding-top","10"),("padding-bottom","5"),("type","bumper"),("report","track")]);        
        add_style(&mut tree, "z/b/", &[("type","bumper"),("report","track")]);
        add_style(&mut tree, "z/a/1", &[("depth","10"),("system","tracking")]);
        add_style(&mut tree, "**", &[("system","tracking")]);
        let style_group = AllotmentStyleGroup::new(StyleTree::new(tree));
        let pending = make_pendings(&["z/a/1","z/a/2","z/a/3","z/b/1","z/b/2","z/b/3"],&[1.,2.,3.],&ranges,&style_group);
        let mut prep = BoxPositionContext::new(None);
        prep.bp_px_converter = Arc::new(BpPxConverter::new_test());
        assert!(make_transformable(&mut prep,&mut pending.iter()).ok().is_some());
        let metadata = prep.state_request.metadata();
        let mut aia = AnswerAllocator::new();
        let mut answer_index = aia.get();
        let transformers = pending.iter().map(|x| prep.plm.transformable(x.name()).make(&answer_index)).collect::<Vec<_>>();
        let descs = transformers.iter().map(|x| x.describe()).collect::<Vec<_>>();
        assert_eq!(6,descs.len());
        println!("{:?}",descs);
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
        let metadata = LocalAllotmentMetadata::new(metadata);
        let mut global_metadata = GlobalAllotmentMetadataBuilder::new();
        metadata.add(&mut global_metadata);
        let metadata = GlobalAllotmentMetadata::new(global_metadata,&mut answer_index);
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
        assert_eq!(Some(&"3".to_string()),b.get("height"));
    }
}
