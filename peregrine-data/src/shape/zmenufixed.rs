use std::collections::HashMap;
use std::rc::Rc;
use super::zmenu::{ ZMenu, ZMenuBlock, ZMenuSequence, ZMenuText, ZMenuItem };
use keyed::{ keyed_handle, KeyedValues };

keyed_handle!(ZMenuKey);

struct ValueSource {
    values: Vec<String>
}

impl ValueSource {
    fn new(name: &str, data: &HashMap<String,Vec<String>>) -> ValueSource {
        let values = data.get(name).map(|x| x.to_vec()).unwrap_or_else(|| vec![]);
        ValueSource {
            values: if values.len() > 0 { values } else { vec!["".to_string()] }
        }
    }

    fn value(&self, index: usize) -> String {
        self.values[index%self.values.len()].to_string()
    }
}

#[cfg_attr(debug_assertions,derive(Debug))]
enum ZMenuBuildText {
    Fixed(String),
    Template(ZMenuKey)
}

impl ZMenuBuildText {
    fn new(text: &ZMenuText, values: &mut KeyedValues<ZMenuKey,ValueSource>, data: &HashMap<String,Vec<String>>) -> ZMenuBuildText {
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
    fn new(item: &ZMenuItem, values: &mut KeyedValues<ZMenuKey,ValueSource>, data: &HashMap<String,Vec<String>>) -> ZMenuBuildItem {
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
pub struct ZMenuFixedBlock {
    pub items: Vec<ZMenuFixedItem>
}

#[cfg_attr(debug_assertions,derive(Debug))]
struct ZMenuBuildBlock(Vec<ZMenuBuildItem>);

impl ZMenuBuildBlock {
    fn new(block: &ZMenuBlock, values: &mut KeyedValues<ZMenuKey,ValueSource>, data: &HashMap<String,Vec<String>>) -> ZMenuBuildBlock {
        ZMenuBuildBlock(block.0.iter().map(|x| ZMenuBuildItem::new(x,values,data)).collect())
    }

    fn value(&self, values: &KeyedValues<ZMenuKey,ValueSource>, index: usize) -> ZMenuFixedBlock {
        ZMenuFixedBlock {
            items: self.0.iter().map(|x| x.value(values,index)).collect()
        }
    }
}

#[cfg_attr(debug_assertions,derive(Debug))]

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
    fn new(seq: &ZMenuSequence, values: &mut KeyedValues<ZMenuKey,ValueSource>, data: &HashMap<String,Vec<String>>) -> ZMenuBuildSequence {
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

#[cfg_attr(debug_assertions,derive(Debug))]
pub struct ZMenuFixed {
    pub sequence: Vec<ZMenuFixedSequence>
}

#[cfg_attr(debug_assertions,derive(Debug))]
struct ZMenuBuild(pub Vec<ZMenuBuildSequence>);

impl ZMenuBuild {
    fn build(zmenu: &ZMenu, data: &HashMap<String,Vec<String>>) -> (ZMenuBuild,KeyedValues<ZMenuKey,ValueSource>) {
        let mut values : KeyedValues<ZMenuKey,ValueSource> = KeyedValues::new();
        let build = ZMenuBuild(zmenu.0.iter().map(|x| ZMenuBuildSequence::new(x,&mut values,data)).collect());
        (build,values)
    }

    fn value(&self, values: &KeyedValues<ZMenuKey,ValueSource>, index: usize) -> ZMenuFixed {
        ZMenuFixed {
            sequence: self.0.iter().map(|x| x.value(values,index)).collect()
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
    pub fn new(zmenu: &ZMenu, data: &HashMap<String,Vec<String>>) -> ZMenuGenerator {
        let (build,values) = ZMenuBuild::build(zmenu,data);
        ZMenuGenerator {
            build: Rc::new(build), values: Rc::new(values)
        }
    }

    pub fn make_proxy(&self, index: usize) -> ZMenuProxy { ZMenuProxy(self.clone(),index) }
    pub fn iter(&self) -> ZMenuProxyIter { ZMenuProxyIter(self.clone(),0) }
}
