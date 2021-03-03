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

struct ZMenuFixedItem {
    text: String,
    markup: Vec<String>
}

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

struct ZMenuFixedBlock(Vec<ZMenuFixedItem>);

struct ZMenuBuildBlock(Vec<ZMenuBuildItem>);

impl ZMenuBuildBlock {
    fn new(block: &ZMenuBlock, values: &mut KeyedValues<ZMenuKey,ValueSource>, data: &HashMap<String,Vec<String>>) -> ZMenuBuildBlock {
        ZMenuBuildBlock(block.0.iter().map(|x| ZMenuBuildItem::new(x,values,data)).collect())
    }

    fn value(&self, values: &KeyedValues<ZMenuKey,ValueSource>, index: usize) -> ZMenuFixedBlock {
        ZMenuFixedBlock(self.0.iter().map(|x| x.value(values,index)).collect())
    }
}

enum ZMenuFixedSequence {
    Item(ZMenuFixedBlock),
    LineBreak
}

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

struct ZMenuFixed(Vec<ZMenuFixedSequence>);
struct ZMenuBuild(Vec<ZMenuBuildSequence>);

impl ZMenuBuild {
    fn build(zmenu: &ZMenu, data: &HashMap<String,Vec<String>>) -> (ZMenuBuild,KeyedValues<ZMenuKey,ValueSource>) {
        let mut values : KeyedValues<ZMenuKey,ValueSource> = KeyedValues::new();
        let build = ZMenuBuild(zmenu.0.iter().map(|x| ZMenuBuildSequence::new(x,&mut values,data)).collect());
        (build,values)
    }

    fn value(&self, values: &KeyedValues<ZMenuKey,ValueSource>, index: usize) -> ZMenuFixed {
        ZMenuFixed(self.0.iter().map(|x| x.value(values,index)).collect())
    }
}

#[derive(Clone)]
pub struct ZMenuGenerator {
    build: Rc<ZMenuBuild>,
    values: Rc<KeyedValues<ZMenuKey,ValueSource>>
}

pub struct ZMenuProxy(ZMenuGenerator,usize);

impl ZMenuProxy {
    fn value(&self) -> ZMenuFixed {
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
    pub(crate) fn new(zmenu: &ZMenu, data: &HashMap<String,Vec<String>>) -> ZMenuGenerator {
        let (build,values) = ZMenuBuild::build(zmenu,data);
        ZMenuGenerator {
            build: Rc::new(build), values: Rc::new(values)
        }
    }

    pub(crate) fn make_proxy(&self, index: usize) -> ZMenuProxy { ZMenuProxy(self.clone(),index) }
    pub(crate) fn iter(&self) -> ZMenuProxyIter { ZMenuProxyIter(self.clone(),0) }
}
