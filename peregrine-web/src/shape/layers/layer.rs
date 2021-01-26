use anyhow::bail;
use std::collections::HashMap;
use std::rc::Rc;
use super::super::core::pingeometry::PinGeometry;
use super::super::core::fixgeometry::FixGeometry;
use super::super::core::tapegeometry::TapeGeometry;
use super::super::core::pagegeometry::PageGeometry;
use super::super::core::directcolourdraw::DirectColourDraw;
use super::super::core::spotcolourdraw::SpotColourDraw;
use crate::webgl::{ ProcessBuilder, SourceInstrs, WebGlCompiler, AccumulatorCampaign };
use super::geometry::{ GeometryAccessor, GeometryAccessorName };
use super::programstore::ProgramStore;
use super::patina::{ PatinaAccessor, PatinaAccessorName };
use peregrine_core::DirectColour;

/* TODO 

Wiggles
macroise
split accumulator
ensure + index
attribute "set" removal
y split bug
y from bottom
layers from core
ordered layers
does everything need context ref?
push up handle resolution via attrib factory (eg spot)
split layer
rearrange accessors
rename accessors

*/

struct SubLayer<'c> {
    process: ProcessBuilder<'c>,
    geometry: GeometryAccessor,
    patina: PatinaAccessor
}

struct SubLayerHolder<'c>(Option<SubLayer<'c>>);

impl<'c> SubLayerHolder<'c> {
    fn new() -> SubLayerHolder<'c> { SubLayerHolder(None) }

    fn make(&mut self, programs: &ProgramStore<'c>, geometry: &GeometryAccessorName, patina: &PatinaAccessorName) -> anyhow::Result<()> {
        let program = programs.get_program(geometry.get_variety(),patina.get_variety())?;
        let process = ProcessBuilder::new(program);
        let geometry = geometry.make_accessor(&process,patina)?;
        let patina = patina.make_accessor(&process)?;
        self.0 = Some(SubLayer { process, geometry, patina });
        Ok(())
    }

    fn get_process_mut(&mut self, programs: &ProgramStore<'c>, geometry: &GeometryAccessorName, patina: &PatinaAccessorName) -> anyhow::Result<&mut ProcessBuilder<'c>> {
        self.make(programs,geometry,patina)?;
        Ok(&mut self.0.as_mut().unwrap().process)
    }

    fn get_geometry(&mut self, programs: &ProgramStore<'c>, geometry: &GeometryAccessorName, patina: &PatinaAccessorName) -> anyhow::Result<&GeometryAccessor> {
        self.make(programs,geometry,patina)?;
        Ok(&self.0.as_mut().unwrap().geometry)
    }

    fn get_patina(&mut self, programs: &ProgramStore<'c>, geometry: &GeometryAccessorName, patina: &PatinaAccessorName) -> anyhow::Result<&PatinaAccessor> {
        self.make(programs,geometry,patina)?;
        Ok(&self.0.as_mut().unwrap().patina)
    }
}

struct GeometrySubLayer<'c> {
    direct: SubLayerHolder<'c>,
    spot: HashMap<DirectColour,SubLayerHolder<'c>>
}

impl<'c> GeometrySubLayer<'c> {
    fn new() -> GeometrySubLayer<'c> {
        GeometrySubLayer {
            direct: SubLayerHolder::new(),
            spot: HashMap::new()
        }
    }

    fn holder(&mut self, patina: &PatinaAccessorName) -> anyhow::Result<&mut SubLayerHolder<'c>> {
        Ok(match &patina{
            PatinaAccessorName::Direct => &mut self.direct,
            PatinaAccessorName::Spot(c) => self.spot.entry(c.clone()).or_insert_with(|| SubLayerHolder::new())
        })
    }

    fn get_process_mut(&mut self, programs: &ProgramStore<'c>, geometry: &GeometryAccessorName, patina: &PatinaAccessorName) -> anyhow::Result<&mut ProcessBuilder<'c>> {
        self.holder(patina)?.get_process_mut(programs,geometry,patina)
    }

    fn get_geometry(&mut self, programs: &ProgramStore<'c>, geometry: &GeometryAccessorName, patina: &PatinaAccessorName) -> anyhow::Result<&GeometryAccessor> {
        self.holder(patina)?.get_geometry(programs,geometry,patina)
    }

    fn get_patina(&mut self, programs: &ProgramStore<'c>, geometry: &GeometryAccessorName, patina: &PatinaAccessorName) -> anyhow::Result<&PatinaAccessor> {
        self.holder(patina)?.get_patina(programs,geometry,patina)
    }
}

pub(crate) struct Layer<'c> {
    programs: &'c ProgramStore<'c>,
    pin: GeometrySubLayer<'c>,
    fix: GeometrySubLayer<'c>,
    tape: GeometrySubLayer<'c>,
    page: GeometrySubLayer<'c>
}

impl<'c> Layer<'c> {
    pub fn new(programs: &'c ProgramStore<'c>) -> Layer<'c> {
        Layer {
            programs,
            pin: GeometrySubLayer::new(),
            fix: GeometrySubLayer::new(),
            tape: GeometrySubLayer::new(),
            page: GeometrySubLayer::new()
        }
    }

    fn holder(&mut self, geometry: &GeometryAccessorName, patina: &PatinaAccessorName) -> anyhow::Result<(&mut GeometrySubLayer<'c>,&'c ProgramStore<'c>)> {
        Ok(match geometry {
            GeometryAccessorName::Pin => (&mut self.pin,&mut self.programs),
            GeometryAccessorName::Fix => (&mut self.fix,&mut self.programs),
            GeometryAccessorName::Tape => (&mut self.tape,&mut self.programs),
            GeometryAccessorName::Page => (&mut self.page,&mut self.programs),
        })
    }

    pub(crate) fn get_process_mut(&mut self, geometry: &GeometryAccessorName, patina: &PatinaAccessorName) -> anyhow::Result<&mut ProcessBuilder<'c>> {
        let (sub,compiler) = self.holder(geometry,patina)?;
        sub.get_process_mut(compiler,geometry,patina)
    }

    fn get_geometry(&mut self, geometry: &GeometryAccessorName, patina: &PatinaAccessorName) -> anyhow::Result<&GeometryAccessor> {
        let (sub,compiler) = self.holder(geometry,patina)?;
       sub.get_geometry(compiler,geometry,patina)
    }

    fn get_patina(&mut self, geometry: &GeometryAccessorName, patina: &PatinaAccessorName) -> anyhow::Result<&PatinaAccessor> {
        let (sub,compiler) = self.holder(geometry,patina)?;
        sub.get_patina(compiler,geometry,patina)
    }

    pub(crate) fn get_pin(&mut self, patina: &PatinaAccessorName) -> anyhow::Result<PinGeometry> {
        let geom = self.get_geometry(&GeometryAccessorName::Pin,patina)?;
        match geom { GeometryAccessor::Pin(x) => Ok(x.clone()), _ => bail!("inconsistent layer") }
    }

    pub(crate) fn get_fix(&mut self, patina: &PatinaAccessorName) -> anyhow::Result<FixGeometry> {
        let geom = self.get_geometry(&GeometryAccessorName::Fix,patina)?;
        match geom { GeometryAccessor::Fix(x) => Ok(x.clone()), _ => bail!("inconsistent layer") }
    }

    pub(crate) fn get_page(&mut self, patina: &PatinaAccessorName) -> anyhow::Result<PageGeometry> {
        let geom = self.get_geometry(&GeometryAccessorName::Page,patina)?;
        match geom { GeometryAccessor::Page(x) => Ok(x.clone()), _ => bail!("inconsistent layer") }
    }

    pub(crate) fn get_tape(&mut self, patina: &PatinaAccessorName) -> anyhow::Result<TapeGeometry> {
        let geom = self.get_geometry(&GeometryAccessorName::Tape,patina)?;
        match geom { GeometryAccessor::Tape(x) => Ok(x.clone()), _ => bail!("inconsistent layer") }
    }

    pub(crate) fn get_direct(&mut self, geometry: &GeometryAccessorName) -> anyhow::Result<DirectColourDraw> {
        let patina = self.get_patina(geometry,&PatinaAccessorName::Direct)?;
        match patina { PatinaAccessor::Direct(x) => Ok(x.clone()), _ => bail!("inconsistent layer") }
    }

    pub(crate) fn get_spot(&mut self, geometry: &GeometryAccessorName, colour: &DirectColour) -> anyhow::Result<SpotColourDraw> {
        let patina = self.get_patina(geometry,&PatinaAccessorName::Spot(colour.clone()))?;
        match patina { PatinaAccessor::Spot(x) => Ok(x.clone()), _ => bail!("inconsistent layer") }
    }

    pub(crate) fn make_campaign(&mut self, geometry: &GeometryAccessorName, patina: &PatinaAccessorName, count: usize, indexes: &[u16]) -> anyhow::Result<AccumulatorCampaign> {
        let process = self.get_process_mut(geometry,patina)?;
        Ok(process.get_accumulator().make_campaign(count,indexes)?)
    }
}
