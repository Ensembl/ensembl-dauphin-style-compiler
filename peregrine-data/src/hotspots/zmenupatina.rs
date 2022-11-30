/* "ZMenu"s are added separately by dauphin. If their boxes overlap then they
 * compose into a single visual ZMenu however all the generating code treats them
 * as separate entities.
 * 
 *   +--------------
 *   | First ZMenuFeature ......
 *   | Still first ZMenuFeature
 *   |
 *   | Second ZMenuFeature......
 *   | Still second ZMenuFreature
 *   | ...
 * 
 * A ZMenuFeature is structured a little like an old-school image gallery
 * being a sequence of items which are arranged horizontally, a bit like 
 * inline blocks, with occasional "breaks" forcing them onto a new line.
 * Each of those items is called ZMenuBlock and the separator a LineBreak.
 * The enum ZMenuBlock/LineBreak is a ZMenuSequence
 * 
 *   | ZMenuBlock1   ZMenuBlock2   ZMenuBlock3
 *   | ZMenuBlock4   ZMenuBlock5   ZMenuBlock6
 * ==
 *   ZMenuBlock1 ZMenuBlock2 ZMenuBlock3 LineBreak ZMenuBlock4 ZMenuBlock5 ZMenuBlock6 
 * ==
 *   seven ZMenuSequences.
 * 
 * A ZMenuBlock should "run" like a sequence of text and not be offset with internal
 * structure. However it has structure in that items can be styled inline as italic,
 * links, etc. It's therefore composed of a sequence of ZMenuItems which include text
 * (literal or placeholder) and markup flags. Each piece of text is a ZMenuText which
 * is either literally text or a placeholder for data.
 * 
 * Because, rightly no one really cares about this internal structure a template mini-
 * language constructs these from a simple string. Items are enclosed in [] and line-
 * breaks indicated with a /. On and off markup strings are in <>...</> (like XML) and
 * template strings are in {}. Any can be backslash-escaped. Templates are the only
 * way to create these things. The above internal structure exists to simplify operations
 * on ZMenuFeatures.
 * 
 * ZMenuToken is a lex token used temporarily internally during parsing and can be ignored.
 */

// TODO hashmap to docs

use anyhow::{ bail };
use peregrine_toolkit::eachorevery::EachOrEvery;
use std::collections::{ HashSet, HashMap };
use std::iter::Peekable;
use std::str::Chars;
use std::sync::Arc;
use crate::{HotspotResult, SpaceBasePoint, LeafStyle};
use super::zmenuitem::ZMenuBuild;

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub enum ZMenuText {
    Fixed(String),
    Template(String)
}

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct ZMenuItem {
    pub text: ZMenuText,
    pub markup: Vec<String>
}

impl ZMenuItem {
    fn new(text: ZMenuText, markup: Vec<String>) -> ZMenuItem {
        ZMenuItem { text, markup }
    }
}

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub(super) struct ZMenuBlock(pub Vec<ZMenuItem>);

#[derive(Clone)]
enum ZMenuToken {
    Text(ZMenuText),
    MarkupOn(String),
    MarkupOff(String)
}

fn flush_any_literal(out: &mut Vec<ZMenuToken>, fixed: &mut String) {
    if fixed.len() > 0 {
        out.push(ZMenuToken::Text(ZMenuText::Fixed(fixed.to_string())));
        fixed.clear();
    }
}

fn get_to_char(chars: &mut Peekable<Chars>, term: char) -> anyhow::Result<String> {
    let mut out = String::new();
    let mut bs = false;
    loop {
        if let Some(c) = chars.next() {
            if bs {
                out.push(c);
                bs = false;
            } else if c == term {
                break;
            } else if c == '\\' {
                bs = true;
            } else {
                out.push(c);
            }
        } else {
            bail!("unexpected EOF");
        }
    }
    Ok(out)
}

fn fmt_zmenu_blocks_lex(chars: &mut Peekable<Chars>) -> anyhow::Result<Vec<ZMenuToken>> {
    let mut out = Vec::new();
    let mut bs = false;
    let mut literal = String::new();
    while let Some(c) = chars.peek() {
        if bs {
            literal.push(chars.next().unwrap());
            bs = false;
        } else {
            match *c {
                '{' => {
                    flush_any_literal(&mut out,&mut literal);
                    chars.next();
                    out.push(ZMenuToken::Text(ZMenuText::Template(get_to_char(chars,'}')?)));        
                },
                '<' => {
                    flush_any_literal(&mut out,&mut literal);
                    chars.next();
                    if chars.peek() == Some(&'/') {
                        chars.next();
                        out.push(ZMenuToken::MarkupOff(get_to_char(chars,'>')?));
                    } else {
                        out.push(ZMenuToken::MarkupOn(get_to_char(chars,'>')?));
                    }        
                },
                ']' => { break; }
                '\\' => { bs = true; },
                _ => { literal.push(chars.next().unwrap()) }
            }
        }
    }
    flush_any_literal(&mut out,&mut literal);
    Ok(out)
}

fn fmt_zmenu_blocks_parse(tags: Vec<ZMenuToken>) -> anyhow::Result<Vec<ZMenuItem>> {
    let mut markup = HashSet::new();
    let mut items = Vec::new();
    for tag in tags {
         match tag {
            ZMenuToken::Text(text) => {
                let markup_vec = markup.iter().cloned().collect();
                items.push(ZMenuItem::new(text.clone(),markup_vec));
            },
            ZMenuToken::MarkupOn(m) => {
                markup.insert(m.to_string());
            },
            ZMenuToken::MarkupOff(m) => {
                if !markup.remove(&m) {
                    bail!("close tag without open!");
                }
            }
        }
    }
    if markup.len() > 0 {
        bail!("unclosed open tag");
    }
    Ok(items)
}

impl ZMenuBlock {
    fn new(chars: &mut Peekable<Chars>) -> anyhow::Result<ZMenuBlock> {
        Ok(ZMenuBlock(fmt_zmenu_blocks_parse(fmt_zmenu_blocks_lex(chars)?)?))
    }
}

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub(super) enum ZMenuSequence {
    Item(ZMenuBlock),
    LineBreak
}

fn fmt_zmenu_sequences(spec: &str) -> anyhow::Result<Vec<ZMenuSequence>> {
    // only [ or / (or dead-ws) allowed!
    let mut chars = spec.chars().peekable();
    let mut out = vec![];
    while let Some(c) = chars.next() {
        match c {
            '[' => {
                out.push(ZMenuSequence::Item(ZMenuBlock::new(&mut chars)?));
                if chars.next() != Some(']') { bail!("expected ]"); }
            },
            '/' => {
                out.push(ZMenuSequence::LineBreak);
            },
            c if c.is_whitespace() => {},
            _ => {
                bail!("unecpected character: {}",c);
            }
        }
    }
    Ok(out)
}

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct ZMenu(pub(super) Vec<ZMenuSequence>);

impl ZMenu {
    pub fn new(spec: &str) -> anyhow::Result<ZMenu> {
        Ok(ZMenu(fmt_zmenu_sequences(spec)?))
    }
}

pub fn zmenu_generator(zmenu: &ZMenu, values: &Vec<(String,EachOrEvery<String>)>) -> Arc<dyn Fn(usize,Option<(SpaceBasePoint<f64,LeafStyle>,SpaceBasePoint<f64,LeafStyle>)>) -> HotspotResult> {
    let mut map_values = HashMap::new();
    for (k,v) in values.iter() {
        map_values.insert(k.to_string(),v.clone());
    }
    let (build,values) = ZMenuBuild::build(zmenu,&map_values);
    Arc::new(move |index,_| {
        HotspotResult::ZMenu(build.value(&values,index))
    })    
}
