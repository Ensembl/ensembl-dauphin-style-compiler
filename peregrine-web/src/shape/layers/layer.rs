use std::rc::Rc;
use super::super::core::paintgeometry::PaintGeometry;
use super::super::core::paintskin::PaintSkin;
use super::super::core::paintmethod::PaintMethod;
use super::pingeometry::PinGeometry;
use super::fixgeometry::FixGeometry;
use super::tapegeometry::TapeGeometry;
use super::pagegeometry::PageGeometry;
use super::directcolourdraw::DirectColourDraw;
use crate::webgl::{ ProcessBuilder, SourceInstrs, WebGlCompiler, AccumulatorCampaign };

/* TODO 

Wiggles
Pullout programs
macroise
split accumulator
ensure + index
attribute "set" removal
y split bug
y from bottom
layers from core
ordered layers

*/

pub struct Layer<'c> {
    compiler: WebGlCompiler<'c>,
    sublayers: Vec<Option<ProcessBuilder<'c>>>,
    pins: Vec<Option<PinGeometry>>,
    fixes: Vec<Option<FixGeometry>>,
    tapes: Vec<Option<TapeGeometry>>,
    pages: Vec<Option<PageGeometry>>,
    directs: Vec<Option<DirectColourDraw>>
}

fn index(geometry: &PaintGeometry, skin: &PaintSkin) -> usize {
    (geometry.to_index()*skin.num_values()+skin.to_index()) as usize
}

impl<'c> Layer<'c> {
    fn ensure_sublayer(&mut self, geometry: &PaintGeometry, skin: &PaintSkin) -> anyhow::Result<()> {
        let idx = index(geometry,skin);
        if self.sublayers[idx].is_none() {
            let mut source = SourceInstrs::new(vec![]);
            source.merge(geometry.to_source());
            source.merge(PaintMethod::Triangle.to_source());
            source.merge(skin.to_source());
            let program = self.compiler.make_program(source)?; // XXX pull out
            self.sublayers[idx] = Some(ProcessBuilder::new(Rc::new(program)));
        }
        Ok(())
    }

    pub(crate) fn make_campaign<'a>(&mut self, geometry: &PaintGeometry, skin: &PaintSkin, count: usize, indexes: &[u16]) -> anyhow::Result<AccumulatorCampaign> {
        let full_idx = index(geometry,skin);
        self.ensure_sublayer(geometry,skin)?;
        let process = self.sublayers[full_idx].as_mut().unwrap();
        let campaign = process.get_accumulator().make_campaign(count,indexes)?;
        Ok(campaign)
    }
    
    pub(crate) fn get_pin(&mut self, skin: &PaintSkin) -> anyhow::Result<PinGeometry> {
        let full_idx = index(&PaintGeometry::Pin,skin);
        self.ensure_sublayer(&PaintGeometry::Pin,skin)?;
        let process = self.sublayers[full_idx].as_mut().unwrap();
        if self.pins[skin.to_index()].is_none() {
            self.pins[skin.to_index()] = Some(PinGeometry::new(process)?);
        }
        Ok(self.pins[skin.to_index()].as_ref().unwrap().clone())
    }

    pub(crate) fn get_fix(&mut self, skin: &PaintSkin) -> anyhow::Result<FixGeometry> {
        let full_idx = index(&PaintGeometry::Fix,skin);
        self.ensure_sublayer(&PaintGeometry::Fix,skin)?;
        let process = self.sublayers[full_idx].as_mut().unwrap();
        if self.fixes[skin.to_index()].is_none() {
            self.fixes[skin.to_index()] = Some(FixGeometry::new(process)?);
        }
        Ok(self.fixes[skin.to_index()].as_ref().unwrap().clone())
    }

    pub(crate) fn get_tape(&mut self, skin: &PaintSkin) -> anyhow::Result<TapeGeometry> {
        let full_idx = index(&PaintGeometry::Tape,skin);
        self.ensure_sublayer(&PaintGeometry::Tape,skin)?;
        let process = self.sublayers[full_idx].as_mut().unwrap();
        if self.tapes[skin.to_index()].is_none() {
            self.tapes[skin.to_index()] = Some(TapeGeometry::new(process)?);
        }
        Ok(self.tapes[skin.to_index()].as_ref().unwrap().clone())
    }

    pub(crate) fn get_page(&mut self, skin: &PaintSkin) -> anyhow::Result<PageGeometry> {
        let full_idx = index(&PaintGeometry::Page,skin);
        self.ensure_sublayer(&PaintGeometry::Page,skin)?;
        let process = self.sublayers[full_idx].as_mut().unwrap();
        if self.pages[skin.to_index()].is_none() {
            self.pages[skin.to_index()] = Some(PageGeometry::new(process)?);
        }
        Ok(self.pages[skin.to_index()].as_ref().unwrap().clone())
    }

    pub(crate) fn get_direct(&mut self, geometry: &PaintGeometry) -> anyhow::Result<DirectColourDraw> {
        let full_idx = index(geometry,&PaintSkin::Colour);
        self.ensure_sublayer(geometry,&PaintSkin::Colour)?;
        let process = self.sublayers[full_idx].as_mut().unwrap();
        if self.directs[geometry.to_index()].is_none() {
            self.directs[geometry.to_index()] = Some(DirectColourDraw::new(process)?);
        }
        Ok(self.directs[geometry.to_index()].as_ref().unwrap().clone())
    }
}
