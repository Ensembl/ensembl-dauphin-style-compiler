use std::{collections::HashMap};
use peregrine_toolkit::error::Error;
use crate::{allotment::{core::{trainstate::CarriageTrainStateSpec, allotmentname::{AllotmentNamePart, AllotmentName}, boxpositioncontext::BoxPositionContext, drawinginfo::DrawingInfo, leafrequest::LeafTransformableMap, boxtraits::ContainerOrLeaf}, boxes::{leaf::{FloatingLeaf}, root::Root}}, LeafRequest, LeafStyle};

struct StyleBuilder<'a> {
    root: &'a mut dyn ContainerOrLeaf,
    leafs_made: HashMap<Vec<String>,FloatingLeaf>,
    dustbin: FloatingLeaf
}

impl<'a> StyleBuilder<'a> {
    fn new(root: &'a mut Root) -> StyleBuilder<'a> {
        let dustbin_name = AllotmentNamePart::new(AllotmentName::new(""));
        StyleBuilder {
            root,
            leafs_made: HashMap::new(),
            dustbin: FloatingLeaf::new(&dustbin_name,&LeafStyle::dustbin(),&DrawingInfo::new())
        }
    }

    fn try_new_leaf(&mut self, pending: &LeafRequest) -> Result<FloatingLeaf,Error> {
        let name = AllotmentNamePart::new(pending.name().clone());
        if name.empty() {
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

pub(crate) fn make_transformable(prep: &mut BoxPositionContext, pendings: &mut dyn Iterator<Item=&LeafRequest>) -> Result<(CarriageTrainStateSpec,LeafTransformableMap),Error> {
    let mut root = Root::new();
    /* Build box tree */
    let mut plm = LeafTransformableMap::new();
    let mut styler = StyleBuilder::new(&mut root);
    for pending in pendings {
        let xformable = styler.try_new_leaf(&pending)?;
        plm.set_transformable(&pending.name(),&xformable);
    }
    drop(styler);
    /* Wire box tree */
    let state_spec = root.full_build(prep);
    Ok((state_spec,plm))
}

#[cfg(test)]
mod test {
    use std::{sync::{Arc, Mutex}, collections::{HashMap}};
    use peregrine_toolkit::{puzzle::{AnswerAllocator}};
    use crate::{allotment::{core::{allotmentname::AllotmentName, boxpositioncontext::BoxPositionContext, trainstate::CarriageTrainStateSpec, boxtraits::ContainerOrLeaf}, stylespec::{stylegroup::AllStylesForProgram, styletreebuilder::StyleTreeBuilder, styletree::StyleTree}, util::{bppxconverter::BpPxConverter, rangeused::RangeUsed}, globals::{allotmentmetadata::{LocalAllotmentMetadata, GlobalAllotmentMetadataBuilder, GlobalAllotmentMetadata}, bumping::{GlobalBumpBuilder, GlobalBump}, trainpersistent::TrainPersistent}, builder::stylebuilder::make_transformable}, LeafRequest, shape::metadata::{AbstractMetadataBuilder}};
    use serde_json::{Value as JsonValue };

    fn make_pendings(names: &[&str], heights: &[f64], pixel_range: &[RangeUsed<f64>], style: &AllStylesForProgram) -> Vec<LeafRequest> {
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
            leaf.set_program_styles(style);
            leaf.drawing_info(|info| {
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
        let style_group = AllStylesForProgram::new(StyleTree::new(tree));
        let pending = make_pendings(&["z/a/1","z/a/2","z/a/3","z/b/1","z/b/2","z/b/3"],&[1.,2.,3.],&[],&style_group);
        let mut prep = BoxPositionContext::new(None,&AbstractMetadataBuilder::new().build());
        let (spec,plm) = make_transformable(&mut prep,&mut pending.iter()).ok().expect("A");
        let mut aia = AnswerAllocator::new();
        let answer_index = aia.get();
        let transformers = pending.iter().map(|x| plm.transformable(x.name()).anchor_leaf(&answer_index).unwrap()).collect::<Vec<_>>();
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
        let style_group = AllStylesForProgram::new(StyleTree::new(tree));
        let pending = make_pendings(&["z/a/1","z/a/2","z/a/3","z/b/1","z/b/2","z/b/3"],&[1.,2.,3.],&[],&style_group);
        let mut prep = BoxPositionContext::new(None,&AbstractMetadataBuilder::new().build());
        let (spec,plm) = make_transformable(&mut prep,&mut pending.iter()).ok().expect("A");
        let mut aia = AnswerAllocator::new();
        let answer_index = aia.get();
        let transformers = pending.iter().map(|x| plm.transformable(x.name()).anchor_leaf(&answer_index).unwrap()).collect::<Vec<_>>();
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
        let mut tree = StyleTreeBuilder::new();
        add_style(&mut tree, "z/a/", &[("padding-top","10"),("padding-bottom","5"),("type","bumper"),("report","track")]);
        add_style(&mut tree, "z/b/", &[("type","bumper"),("report","track")]);
        add_style(&mut tree, "z/a/1", &[("depth","10"),("system","tracking")]);
        add_style(&mut tree, "**", &[("system","tracking")]);
        let style_group = AllStylesForProgram::new(StyleTree::new(tree));
        let pending = make_pendings(&["z/a/1","z/a/2","z/a/3","z/b/1","z/b/2","z/b/3"],&[1.,2.,3.],&ranges,&style_group);
        let mut prep = BoxPositionContext::new(None,&AbstractMetadataBuilder::new().build());
        prep.bp_px_converter = Arc::new(BpPxConverter::new_test());
        let (spec,plm) = make_transformable(&mut prep,&mut pending.iter()).ok().expect("A");
        let metadata = prep.state_request.metadata();
        let mut aia = AnswerAllocator::new();
        let mut answer_index = aia.get();
        let ctss = CarriageTrainStateSpec::new(&prep.state_request);
        let tp = Arc::new(Mutex::new(TrainPersistent::new()));
        let mut builder = GlobalBumpBuilder::new();
        ctss.bump().add(&mut builder);
        GlobalBump::new(builder,&mut answer_index,&tp);
        let transformers = pending.iter().map(|x| plm.transformable(x.name()).anchor_leaf(&answer_index).unwrap()).collect::<Vec<_>>();
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
