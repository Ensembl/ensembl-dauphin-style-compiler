use std::{collections::HashMap, sync::Arc};
use peregrine_toolkit::{error::Error, puzzle::{StaticValue, derived, StaticAnswer}};
use crate::{allotment::{core::{allotmentname::{AllotmentName, allotmentname_hashmap, AllotmentNameHashMap}, boxpositioncontext::BoxPositionContext, leafshapebounds::LeafShapeBounds}, containers::{root::Root}, style::{leafstyle::LeafStyle, styletree::StyleTree}, leafs::{floating::FloatingLeaf, anchored::AnchoredLeaf}, util::rangeused::RangeUsed}, LeafRequest, CoordinateSystem, shape::metadata::AllotmentMetadataEntry };

pub(crate) struct BuildSize {
    pub name: AllotmentName,
    pub height: StaticValue<f64>,
    pub range: RangeUsed<f64>,
    pub metadata: Vec<AllotmentMetadataEntry>
}

impl BuildSize {
    pub(crate) fn to_value(&self) -> StaticValue<(AllotmentName,f64,RangeUsed<f64>)> {
        let name = self.name.clone();
        let range = self.range.clone();
        derived(self.height.clone(),move |h| {
            (name.clone(),h,range.clone())
        })
    }
}

pub(crate) trait ContainerOrLeaf {
    fn coordinate_system(&self) -> &CoordinateSystem;
    fn build(&mut self, prep: &mut BoxPositionContext) -> BuildSize;
    fn locate(&mut self, prep: &mut BoxPositionContext, top: &StaticValue<f64>);
    fn name(&self) -> &AllotmentName;
    fn priority(&self) -> i64;
    fn anchor_leaf(&self, answer_index: &StaticAnswer) -> Option<AnchoredLeaf>;
    fn get_leaf(&mut self, pending: &LeafRequest, cursor: usize, styles: &Arc<StyleTree>) -> FloatingLeaf;
}

struct StyleBuilder<'a> {
    root: &'a mut dyn ContainerOrLeaf,
    leafs_made: HashMap<Vec<String>,FloatingLeaf>,
    dustbin: FloatingLeaf
}

impl<'a> StyleBuilder<'a> {
    fn new(root: &'a mut Root) -> StyleBuilder<'a> {
        let dustbin_name = AllotmentName::new("");
        StyleBuilder {
            root,
            leafs_made: HashMap::new(),
            dustbin: FloatingLeaf::new(&dustbin_name,&LeafStyle::dustbin(),&LeafShapeBounds::new())
        }
    }

    fn add(&mut self, pending: &LeafRequest) -> Result<FloatingLeaf,Error> {
        let name = pending.name().clone();
        if name.is_dustbin() {
            return Ok(self.dustbin.clone());
        }
        let sequence = name.sequence().to_vec();
        Ok(if let Some(leaf) = self.leafs_made.get(&sequence) {
            leaf.clone()
        } else {
            let out = self.root.get_leaf(pending,0,&pending.program_styles());
            self.leafs_made.insert(sequence,out.clone());
            out.clone()
        })
    }
}

pub(crate) fn make_transformable(pendings: &mut dyn Iterator<Item=&LeafRequest>) -> Result<(Root,AllotmentNameHashMap<FloatingLeaf>),Error> {
    let mut root = Root::new();
    let mut styler = StyleBuilder::new(&mut root);
    let mut leaf_map = allotmentname_hashmap();
    for pending in pendings {
        leaf_map.insert(pending.name().clone(),styler.add(&pending)?);
    }
    Ok((root,leaf_map))
}

#[cfg(test)]
mod test {
    use std::{sync::{Arc, Mutex}, collections::{HashMap}};
    use peregrine_toolkit::{puzzle::{AnswerAllocator}};
    use crate::{allotment::{style::styletree::StyleTree, layout::stylebuilder::ContainerOrLeaf}, globals::{allotmentmetadata::{LocalAllotmentMetadata, GlobalAllotmentMetadataBuilder}, trainstate::CarriageTrainStateSpec}, GlobalAllotmentMetadata};
    use crate::{allotment::{core::{allotmentname::AllotmentName, boxpositioncontext::BoxPositionContext}, util::{bppxconverter::BpPxConverter, rangeused::RangeUsed}, layout::stylebuilder::make_transformable}, LeafRequest };
    use serde_json::{Value as JsonValue };
    use crate::globals::{bumping::{GlobalBumpBuilder, GlobalBump}, trainpersistent::TrainPersistent};

    fn make_pendings(names: &[&str], heights: &[f64], pixel_range: &[RangeUsed<f64>], style: &StyleTree) -> Vec<LeafRequest> {
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
            leaf.shape_bounds(|info| {
                info.merge_max_y(*height);
                if let Some(ref mut pixel_range) = pixel_range_iter {
                    info.merge_pixel_range(pixel_range.next().unwrap());
                }    
            });
            out.push(leaf);
        }
        out
    }

    fn add_style(group: &mut StyleTree, name: &str, values: &[(&str,&str)]) {
        let mut values_hash = vec![];
        for (k,v) in values {
            values_hash.push((k.to_string(),v.to_string()));
        }
        group.add(name,values_hash);
    }

    // XXX generic specs
    // XXX errors

    #[test]
    fn allotment_smoke() {
        let mut tree = StyleTree::new();
        add_style(&mut tree, "z/a/", &[("padding-top","10"),("padding-bottom","5")]);
        add_style(&mut tree, "z/a/1", &[("depth","10"),("coordinate-system","window")]);
        let pending = make_pendings(&["z/a/1","z/a/2","z/a/3","z/b/1","z/b/2","z/b/3"],&[1.,2.,3.],&[],&tree);
        let mut prep = BoxPositionContext::new(None);
        let (_spec,plm) = make_transformable(&mut pending.iter()).ok().expect("A");
        let mut aia = AnswerAllocator::new();
        let answer_index = aia.get();
        let transformers = pending.iter().map(|x| plm.get(x.name()).unwrap().anchor_leaf(&answer_index).unwrap()).collect::<Vec<_>>();
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
        let mut tree = StyleTree::new();
        add_style(&mut tree, "z/a/", &[("padding-top","10"),("padding-bottom","5"),("type","overlay")]);        
        add_style(&mut tree, "z/a/1", &[("depth","10"),("coordinate-system","window")]);
        let pending = make_pendings(&["z/a/1","z/a/2","z/a/3","z/b/1","z/b/2","z/b/3"],&[1.,2.,3.],&[],&tree);
        let mut prep = BoxPositionContext::new(None);
        let (_spec,plm) = make_transformable(&mut pending.iter()).ok().expect("A");
        let mut aia = AnswerAllocator::new();
        let answer_index = aia.get();
        let transformers = pending.iter().map(|x| plm.get(x.name()).unwrap().anchor_leaf(&answer_index).unwrap()).collect::<Vec<_>>();
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

    fn check_metadata(a: &HashMap<String,JsonValue>, key: &str, cmp: &str) {
        let cmp : JsonValue = serde_json::from_str(cmp).expect("bad cmp");
        if let Some(value) = a.get(key) {
            assert_eq!(cmp,value.clone())
        }
    }

    #[test]
    fn allotment_bumper() {
        let ranges = [
            RangeUsed::Part(0.,3.),
            RangeUsed::Part(2.,6.),
            RangeUsed::Part(4.,10.),
            RangeUsed::Part(0.,2.),
            RangeUsed::Part(2.,4.),
            RangeUsed::Part(4.,6.)
        ];
        let mut tree = StyleTree::new();
        add_style(&mut tree, "z/a/", &[("padding-top","10"),("padding-bottom","5"),("type","bumper"),("report","track")]);
        add_style(&mut tree, "z/b/", &[("type","bumper"),("report","track")]);
        add_style(&mut tree, "z/a/1", &[("depth","10"),("system","tracking")]);
        add_style(&mut tree, "**", &[("system","tracking")]);
        let pending = make_pendings(&["z/a/1","z/a/2","z/a/3","z/b/1","z/b/2","z/b/3"],&[1.,2.,3.],&ranges,&tree);
        let mut prep = BoxPositionContext::new(None);
        prep.bp_px_converter = Arc::new(BpPxConverter::new_test());
        let (_spec,plm) = make_transformable(&mut pending.iter()).ok().expect("A");
        let metadata = prep.state_request.metadata();
        let mut aia = AnswerAllocator::new();
        let mut answer_index = aia.get();
        let ctss = CarriageTrainStateSpec::new(&prep.state_request);
        let tp = Arc::new(Mutex::new(TrainPersistent::new()));
        let mut builder = GlobalBumpBuilder::new();
        ctss.bump().add(&mut builder);
        GlobalBump::new(builder,&mut answer_index,&tp);
        let transformers = pending.iter().map(|x| plm.get(x.name()).unwrap().anchor_leaf(&answer_index).unwrap()).collect::<Vec<_>>();
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
        let (a,b) = 
            if a.get("offset").map(|x| x.to_string()) == Some("0.0".to_string()) { 
                (a,b) 
            } else if b.get("offset").map(|x| x.to_string()) == Some("0.0".to_string()) { 
                (b,a)
            } else {
                println!("A {:?} B {:?}",a.get("offset").map(|x| x.to_string()),b.get("offset").map(|x| x.to_string()));
                assert!(false);
                panic!();
            };
        check_metadata(a,"type","\"track\"");
        check_metadata(a,"offset","0.0");
        check_metadata(a,"height","21.0");
        check_metadata(b,"type","\"track\"");
        check_metadata(b,"offset","21.0");
        check_metadata(b,"height","3.0");
    }
}
