/* 
 *  See the NOTICE file distributed with this work for additional information
 *  regarding copyright ownership.
 *  
 *  Licensed under the Apache License, Version 2.0 (the "License"); you may 
 *  not use this file except in compliance with the License. You may obtain a
 *  copy of the License at http://www.apache.org/licenses/LICENSE-2.0
 *  
 *  Unless required by applicable law or agreed to in writing, software
 *  distributed under the License is distributed on an "AS IS" BASIS, WITHOUT 
 *  WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *  
 *  See the License for the specific language governing permissions and
 *  limitations under the License.
 */

 use anyhow::bail;
use std::cell::{ RefCell, Ref };
use std::collections::HashMap;
use std::rc::Rc;
use crate::command::Identifier;
use crate::types::{ FullType, ComplexPath, VectorRegisters };
use crate::util::DauphinError;

#[derive(Debug)]
pub enum XStructure<T> {
    Simple(Rc<RefCell<T>>),
    Vector(Rc<XStructure<T>>),
    Struct(Identifier,HashMap<String,Rc<XStructure<T>>>),
    Enum(Identifier,Vec<String>,HashMap<String,Rc<XStructure<T>>>,Rc<RefCell<T>>),
}

impl<T> Clone for XStructure<T> {
    fn clone(&self) -> Self {
        match self {
            XStructure::Simple(t) => XStructure::Simple(t.clone()),
            XStructure::Vector(v) => XStructure::Vector(v.clone()),
            XStructure::Struct(id,map) => XStructure::Struct(id.clone(),map.clone()),
            XStructure::Enum(id,order,map,disc) => XStructure::Enum(id.clone(),order.clone(),map.clone(),disc.clone())
        }
    }
}


impl<T> XStructure<T> {
    pub fn derive<F,U>(&self, cb: &mut F) -> anyhow::Result<XStructure<U>> where F: FnMut(&T) -> anyhow::Result<U> {
        Ok(match self {
            XStructure::Simple(t) => XStructure::Simple(Rc::new(RefCell::new(cb(&t.borrow())?))),
            XStructure::Vector(v) => XStructure::Vector(Rc::new(v.derive(cb)?)),
            XStructure::Struct(id,map) => {
                let map : anyhow::Result<HashMap<_,_>> = map.iter().map(|(k,v)| Ok((k.to_string(),Rc::new(v.derive(cb)?)))).collect();
                XStructure::Struct(id.clone(),map?)
            },
            XStructure::Enum(id,order,map,disc) => {
                let map : anyhow::Result<HashMap<_,_>> = map.iter().map(|(k,v)| Ok((k.to_string(),Rc::new(v.derive(cb)?)))).collect();
                XStructure::Enum(id.clone(),order.clone(),map?,Rc::new(RefCell::new(cb(&disc.borrow())?)))
            }
        })
    }

    pub fn any(&self) -> Ref<T> {
        match self {
            XStructure::Vector(inner) => inner.any(),
            XStructure::Struct(_,kvs) => kvs.iter().next().unwrap().1.any(),
            XStructure::Enum(_,_,kvs,_) => kvs.iter().next().unwrap().1.any(),
            XStructure::Simple(t) => t.borrow()
        }
    }

    fn max_rec_choose<V,I,U>(&self, it: I) -> Option<(V,U)> where I: Iterator<Item=Option<(V,U)>>, V: Ord {
        let mut out = None;
        for val in it {
            if let Some((v,t)) = val {
                let mut beaten = false;
                if let Some((ref v_max,_)) = out {
                    if v <= *v_max {
                        beaten = true;
                    }
                }
                if !beaten {
                    out = Some((v,t));
                }
            }
        }
        out
    }

    fn max_rec<F,V>(&self, f: &F, default: &V) -> Option<(V,Ref<T>)> where F: Fn(&T) -> V, V: Ord+Clone {
        match self {
            XStructure::Vector(inner) => inner.max_rec(f,default),
            XStructure::Struct(_,kvs) => self.max_rec_choose(kvs.values().map(|x| x.max_rec(f,default))),
            XStructure::Enum(_,_,kvs,_) => self.max_rec_choose(kvs.values().map(|x| x.max_rec(f,default))),
            XStructure::Simple(t) => Some((f(&t.as_ref().borrow()),t.borrow()))
        }
    }

    pub fn max<F,V>(&self, f: F, default: V) -> Option<Ref<T>> where F: Fn(&T) -> V, V: Ord+Clone {
        self.max_rec(&f,&default).map(|x| x.1)
    }
}

#[derive(Debug)]
pub struct XPath<T>(Vec<XPathEl>,Rc<RefCell<T>>);

impl<T> Clone for XPath<T> {
    fn clone(&self) -> Self {
        XPath(self.0.clone(),self.1.clone())
    }
}

#[derive(Debug,Clone)]
pub enum XPathEl {
    Vector(),
    Part(Identifier,String)
}

fn to_xpath<T>(cp: &ComplexPath, vr: T) -> anyhow::Result<XPath<T>> {
    let mut out = vec![];
    let mut name = cp.get_name().ok_or_else(|| DauphinError::internal(file!(),line!()) /* cannot convert anon */)?.iter();
    let mut cursor = cp.get_breaks().iter().peekable();
    while let Some(vecs) = cursor.next() {
        for _ in 0..*vecs {
            out.push(XPathEl::Vector());
        }
        if cursor.peek().is_some() {
            let (obj,field) = name.next().ok_or_else(|| DauphinError::internal(file!(),line!()) /* bad path */)?;
            out.push(XPathEl::Part(obj.clone(),field.to_string()));
        }
    }
    Ok(XPath(out,Rc::new(RefCell::new(vr))))
}

fn enum_split<T>(paths: &[XPath<T>]) -> (Vec<XPath<T>>,Option<XPath<T>>) {
    let mut disc = None;
    let mut rest = vec![];
    for path in paths.iter() {
        if path.0.len() == 0 {
            disc = Some(path.clone());
        } else {
            rest.push(path.clone());
        }
    }
    (rest,disc)
}

fn convert<T>(paths: &[XPath<T>]) -> anyhow::Result<XStructure<T>> {
    /* Is it a simple path? If so, exit */
    if paths.iter().filter(|x| x.0.len()!=0).count() == 0 {
        return Ok(XStructure::Simple(paths[0].1.clone()));
    }
    /* Not a simple path, so a vec, enum, or struct */
    /* All XPaths at the head this level should be Vector or Part, no mixtures.
     * Extract heads into "names" a vec of their XPathEl members or None (if vector).
     */
    let (paths,disc) = enum_split(paths);
    let mut paths : Vec<XPath<T>> = paths.iter().map(|x| x.clone()).collect();
    let heads : Vec<XPathEl> = paths.iter_mut().map(|x| x.0.remove(0)).collect();
    let names : Option<Vec<(Identifier,String)>> = heads.iter().map(|x| {
        if let XPathEl::Part(x,y) = x { Some((x.clone(),y.clone())) } else { None }
    }).collect();
    if let Some(names) = names {
        /* We have a struct or enum */
        let mut mapping = HashMap::new();
        let mut obj_name = None;
        let mut name_order = vec![];
        for (i,name) in names.iter().enumerate() {
            if !mapping.contains_key(&name.1) {
                mapping.insert(name.1.clone(),vec![]);
                name_order.push(name.1.clone());
            }
            mapping.get_mut(&name.1).unwrap().push(paths[i].clone());
            obj_name = Some(name.0.clone());
        }
        let obj_name = obj_name.as_ref().ok_or_else(|| DauphinError::internal(file!(),line!()) /* empty arm in sig */)?.clone();
        let mut entries = HashMap::new();
        for (field,members) in mapping.iter() {
            entries.insert(field.clone(),Rc::new(convert(members)?));
        }
        if let Some(disc) = disc {
            Ok(XStructure::Enum(obj_name,name_order,entries,disc.1))
        } else {
            Ok(XStructure::Struct(obj_name,entries))
        }
    } else {
        /* We have a vector. Recurse with remaining path components. */
        Ok(XStructure::Vector(Rc::new(convert(&paths)?)))
    }
}

pub fn to_xstructure(sig: &FullType) -> anyhow::Result<XStructure<VectorRegisters>> {
    let mut xpaths = vec![];
    for (cp,vr) in sig.iter() {
        xpaths.push(to_xpath(cp,vr)?);
    }
    Ok(convert(&xpaths)?.derive(&mut (|x| Ok((*x).clone())))?)
}

fn map_xstructure_vr(out: &mut [usize], vr: &VectorRegisters, mapping: &[usize]) -> anyhow::Result<()> {
    for (i,r) in mapping.iter().enumerate() {
        out[*r] = if i == 0 {
            vr.data_pos()
        } else if i%2 == 0 {
            vr.length_pos((i-2)/2)?
        } else {
            vr.offset_pos((i-1)/2)?
        };
    }
    Ok(())
}

pub fn map_xstructure(out: &mut [usize], xs: &XStructure<VectorRegisters>, mapping: &XStructure<Vec<usize>>) -> anyhow::Result<()> {
    match (xs,mapping) {
        (XStructure::Simple(vr),XStructure::Simple(mp)) => map_xstructure_vr(out,&vr.borrow(),&mp.borrow())?,
        (XStructure::Vector(vr),XStructure::Vector(mp)) => map_xstructure(out,vr.as_ref(),mp.as_ref())?,
        (XStructure::Struct(_,kv_a),XStructure::Struct(_,kv_b)) => {
            for (k_b,v_b) in kv_b.iter() {
                if let Some(v_a) = kv_a.get(k_b) {
                    map_xstructure(out,v_a,v_b)?;
                }
            }
        },
        (XStructure::Enum(_,_,kv_a,d_a),XStructure::Enum(_,_,kv_b,d_b)) => {
            for (k_b,v_b) in kv_b.iter() {
                if let Some(v_a) = kv_a.get(k_b) {
                    map_xstructure(out,v_a,v_b)?;
                }
            }
            out[d_b.borrow()[0]] = d_a.borrow().data_pos();
        },
        _ => bail!("mismatched xstructures!")
    }
    Ok(())
}