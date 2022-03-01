use std::collections::HashMap;
use std::collections::HashSet;
use std::rc::Rc;
use crate::EachOrEvery;

use super::zmenu::{ ZMenu, ZMenuBlock, ZMenuSequence, ZMenuText, ZMenuItem };
use keyed::{ keyed_handle, KeyedValues };
use serde_json::Number;
use serde_json::Value as JSONValue;
use serde_json::Map as JSONMap;
use serde_json::json;

keyed_handle!(ZMenuKey);

#[cfg_attr(debug_assertions,derive(Debug))]
struct ValueSource {
    values: EachOrEvery<String>
}

impl ValueSource {
    fn new(name: &str, data: &HashMap<String,EachOrEvery<String>>) -> ValueSource {
        let values = data.get(name).cloned().unwrap_or_else(|| EachOrEvery::every("".to_string()));
        ValueSource { values }
    }

    fn value(&self, index: usize) -> String {
        self.values.get(index).map(|x| x.as_str()).unwrap_or_else(|| "").to_string()
    }
}

#[cfg_attr(debug_assertions,derive(Debug))]
enum ZMenuBuildText {
    Fixed(String),
    Template(ZMenuKey)
}

impl ZMenuBuildText {
    fn new(text: &ZMenuText, values: &mut KeyedValues<ZMenuKey,ValueSource>, data: &HashMap<String,EachOrEvery<String>>) -> ZMenuBuildText {
        match text {
            ZMenuText::Fixed(s) => ZMenuBuildText::Fixed(s.to_string()),
            ZMenuText::Template(s) => ZMenuBuildText::Template(values.add(s,ValueSource::new(s,data)))
        }
    }

    fn value(&self, values: &KeyedValues<ZMenuKey,ValueSource>, index: usize) -> String {
        match self {
            ZMenuBuildText::Fixed(s) => s.to_string(),
            ZMenuBuildText::Template(s) => values.data().get(&s).value(index)
        }
    }
}

#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(Clone)]
pub struct ZMenuFixedItem {
    pub text: String,
    pub markup: Vec<String>
}

#[cfg_attr(debug_assertions,derive(Debug))]
struct ZMenuBuildItem {
    text: ZMenuBuildText,
    markup: Vec<String>
}

impl ZMenuBuildItem {
    fn new(item: &ZMenuItem, values: &mut KeyedValues<ZMenuKey,ValueSource>, data: &HashMap<String,EachOrEvery<String>>) -> ZMenuBuildItem {
        ZMenuBuildItem {
            text: ZMenuBuildText::new(&item.text,values,data),
            markup: item.markup.clone()
        }
    }

    fn value(&self, values: &KeyedValues<ZMenuKey,ValueSource>, index: usize) -> ZMenuFixedItem {
        ZMenuFixedItem {
            text: self.text.value(values,index),
            markup: self.markup.to_vec()
        }
    }
}

#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(Clone)]
pub struct ZMenuFixedBlock {
    pub items: Vec<ZMenuFixedItem>
}

#[cfg_attr(debug_assertions,derive(Debug))]
struct ZMenuBuildBlock(Vec<ZMenuBuildItem>);

impl ZMenuBuildBlock {
    fn new(block: &ZMenuBlock, values: &mut KeyedValues<ZMenuKey,ValueSource>, data: &HashMap<String,EachOrEvery<String>>) -> ZMenuBuildBlock {
        ZMenuBuildBlock(block.0.iter().map(|x| ZMenuBuildItem::new(x,values,data)).collect())
    }

    fn value(&self, values: &KeyedValues<ZMenuKey,ValueSource>, index: usize) -> ZMenuFixedBlock {
        ZMenuFixedBlock {
            items: self.0.iter().map(|x| x.value(values,index)).collect()
        }
    }
}

#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(Clone)]
pub enum ZMenuFixedSequence {
    Item(ZMenuFixedBlock),
    LineBreak
}

#[cfg_attr(debug_assertions,derive(Debug))]
enum ZMenuBuildSequence {
    Item(ZMenuBuildBlock),
    LineBreak
}

impl ZMenuBuildSequence {
    fn new(seq: &ZMenuSequence, values: &mut KeyedValues<ZMenuKey,ValueSource>, data: &HashMap<String,EachOrEvery<String>>) -> ZMenuBuildSequence {
        match seq {
            ZMenuSequence::Item(block) => ZMenuBuildSequence::Item(ZMenuBuildBlock::new(block,values,data)),
            ZMenuSequence::LineBreak => ZMenuBuildSequence::LineBreak
        }
    }

    fn value(&self, values: &KeyedValues<ZMenuKey,ValueSource>, index: usize) -> ZMenuFixedSequence {
        match self {
            ZMenuBuildSequence::Item(block) => ZMenuFixedSequence::Item(block.value(values,index)),
            ZMenuBuildSequence::LineBreak => ZMenuFixedSequence::LineBreak
        }
    }
}

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct ZMenuFixed {
    pub sequence: Vec<ZMenuFixedSequence>,
    pub metadata: HashMap<String,String>
}

fn zmenu_item_to_json(zmenu: &ZMenuFixedItem) -> JSONValue {
    json!({
        "text": JSONValue::String(zmenu.text.to_string()),
        "markup": JSONValue::Array(zmenu.markup.iter().map(|x| JSONValue::String(x.to_string())).collect())
    })
}

fn zmenu_fixed_block_to_json(zmenu: &ZMenuFixedBlock) -> JSONValue {
    json!({
        "type": "block",
        "items": JSONValue::Array(zmenu.items.iter().map(|z| zmenu_item_to_json(z)).collect())
    })
}

fn assemble_lines(data: &[ZMenuFixedSequence]) -> Vec<JSONValue> {
    if data.len() == 0 { return vec![]; }
    let mut ret = vec![vec![]];
    for z in data {
        match z {
            ZMenuFixedSequence::Item(block) => {
                ret.last_mut().unwrap().push(zmenu_fixed_block_to_json(block));
            },
            ZMenuFixedSequence::LineBreak => {
                ret.push(vec![]);
            }
        }
    }
    ret.drain(..).map(|x| JSONValue::Array(x)).collect::<Vec<_>>()
}

fn zmenu_fixed_metadata_to_json(zmenu: &ZMenuFixed) -> JSONValue {
    let mut metadata = JSONMap::new();
    for (k,v) in zmenu.metadata.iter() {
        metadata.insert(k.to_string(),JSONValue::String(v.to_string()));
    }
    JSONValue::Object(metadata)
}

fn zmenu_fixed_to_json(zmenu: &ZMenuFixed) -> JSONValue {
    json!({
        "metadata": zmenu_fixed_metadata_to_json(&zmenu),
        "data": JSONValue::Array(assemble_lines(&zmenu.sequence))
    })
}

pub fn zmenu_fixed_vec_to_json(zmenus: &[ZMenuFixed]) -> JSONValue {
    JSONValue::Array(zmenus.iter().map(|z| zmenu_fixed_to_json(z)).collect())
}

fn metadata_hashable(zmenu: &ZMenuFixed) -> Vec<(&String,&String)> {
    let mut out = vec![];
    for (key,value) in &zmenu.metadata {
        out.push((key,value));
    }
    out.sort();
    out
}

fn deduplicate_variety(zmenus: &mut Vec<&ZMenuFixed>) {
    let mut out = vec![];
    let mut seen = HashSet::new();
    for zmenu in zmenus.drain(..) {
        let hash = metadata_hashable(zmenu);
        if !seen.contains(&hash) {
            out.push(zmenu);
            seen.insert(hash);
        }
    }
    *zmenus = out;
}

pub fn zmenu_fixed_vec_to_json_split(zmenus: &[ZMenuFixed]) -> (JSONValue,JSONValue) {
    let mut contents = vec![];
    let mut varieties = vec![];
    for zmenu in zmenus {
        let target = if zmenu.sequence.len() == 0 { &mut varieties } else { &mut contents };
        target.push(zmenu);
    }
    deduplicate_variety(&mut varieties);
    (JSONValue::Array(varieties.iter().map(|z| zmenu_fixed_metadata_to_json(z)).collect()),
     JSONValue::Array(contents.iter().map(|z| zmenu_fixed_to_json(z)).collect()))
}

pub fn zmenu_to_json(x: f64, y: f64, zmenus: &[ZMenuFixed]) -> JSONValue {
    let mut root = JSONMap::new();
    let (variety,content) = zmenu_fixed_vec_to_json_split(zmenus);
    root.insert("x".to_string(),JSONValue::Number(Number::from_f64(x).unwrap()));
    root.insert("y".to_string(),JSONValue::Number(Number::from_f64(y).unwrap()));
    root.insert("content".to_string(),content);
    root.insert("variety".to_string(),variety);

    JSONValue::Object(root)
}

#[cfg_attr(debug_assertions,derive(Debug))]
struct ZMenuBuild{
    data: Vec<ZMenuBuildSequence>,
    metadata: HashMap<String,ValueSource>
}

impl ZMenuBuild {
    fn build(zmenu: &ZMenu, data: &HashMap<String,EachOrEvery<String>>) -> (ZMenuBuild,KeyedValues<ZMenuKey,ValueSource>) {
        let metadata = data.iter().map(|(k,v)| (k.to_string(),ValueSource::new(k,data))).collect::<HashMap<_,_>>();
        let mut values : KeyedValues<ZMenuKey,ValueSource> = KeyedValues::new();
        let build = ZMenuBuild {
            metadata,
            data: zmenu.0.iter().map(|x| ZMenuBuildSequence::new(x,&mut values,data)).collect()
        };
        (build,values)
    }

    fn metadata(&self, index: usize) -> HashMap<String,String> {
        let mut out = HashMap::new();
        for (key,value) in self.metadata.iter() {
            out.insert(key.to_string(),value.value(index));
        }
        out
    }

    fn value(&self, values: &KeyedValues<ZMenuKey,ValueSource>, index: usize) -> ZMenuFixed {
        ZMenuFixed {
            sequence: self.data.iter().map(|x| x.value(values,index)).collect(),
            metadata: self.metadata(index)
        }
    }
}

#[derive(Clone)]
pub struct ZMenuGenerator {
    build: Rc<ZMenuBuild>,
    values: Rc<KeyedValues<ZMenuKey,ValueSource>>
}

pub struct ZMenuProxy(ZMenuGenerator,usize);

impl ZMenuProxy {
    pub fn value(&self) -> ZMenuFixed {
        self.0.build.value(&self.0.values,self.1)
    }
}

pub struct ZMenuProxyIter(ZMenuGenerator,usize);

impl Iterator for ZMenuProxyIter {
    type Item = ZMenuProxy;

    fn next(&mut self) -> Option<ZMenuProxy> {
        let out = self.0.make_proxy(self.1);
        self.1 += 1;
        Some(out)
    }
}

impl ZMenuGenerator {
    pub fn new(zmenu: &ZMenu, data: &HashMap<String,EachOrEvery<String>>) -> ZMenuGenerator {
        let (build,values) = ZMenuBuild::build(zmenu,data);
        ZMenuGenerator {
            build: Rc::new(build), values: Rc::new(values)
        }
    }

    pub fn make_proxy(&self, index: usize) -> ZMenuProxy { ZMenuProxy(self.clone(),index) }
    pub fn iter(&self) -> ZMenuProxyIter { ZMenuProxyIter(self.clone(),0) }
}
