use anyhow::bail;
use std::collections::HashMap;
use super::super::core::pingeometry::PinGeometry;
use super::super::core::fixgeometry::FixGeometry;
use super::super::core::tapegeometry::TapeGeometry;
use super::super::core::pagegeometry::PageGeometry;
use super::super::core::directcolourdraw::DirectColourDraw;
use super::super::core::spotcolourdraw::SpotColourDraw;
use crate::webgl::{ ProtoProcess, AccumulatorCampaign };
use super::geometry::{ GeometryProcess, GeometryProcessName };
use super::programstore::ProgramStore;
use super::patina::{ PatinaProcess, PatinaProcessName };
use peregrine_core::DirectColour;

/* TODO 

Wiggles
macroise
ensure + index
y split bug
y from bottom
layers from core
ordered layers

*/

struct SubLayer<'c> {
    process: ProtoProcess<'c>,
    geometry: GeometryProcess,
    patina: PatinaProcess
}

struct SubLayerHolder<'c>(Option<SubLayer<'c>>);

impl<'c> SubLayerHolder<'c> {
    fn new() -> SubLayerHolder<'c> { SubLayerHolder(None) }

    fn make(&mut self, programs: &ProgramStore<'c>, geometry: &GeometryProcessName, patina: &PatinaProcessName) -> anyhow::Result<()> {
        let program_store_entry = programs.get_program(geometry.get_program_name(),patina.get_program_name())?;
        let process = ProtoProcess::new(program_store_entry.program().clone());
        let geometry = program_store_entry.get_geometry().make_geometry_process(&process,patina)?;
        let patina = program_store_entry.get_patina().make_patina_process(&process,patina)?;
        self.0 = Some(SubLayer { process, geometry, patina });
        Ok(())
    }

    fn get_process_mut(&mut self, programs: &ProgramStore<'c>, geometry: &GeometryProcessName, patina: &PatinaProcessName) -> anyhow::Result<&mut ProtoProcess<'c>> {
        self.make(programs,geometry,patina)?;
        Ok(&mut self.0.as_mut().unwrap().process)
    }

    fn get_geometry(&mut self, programs: &ProgramStore<'c>, geometry: &GeometryProcessName, patina: &PatinaProcessName) -> anyhow::Result<&GeometryProcess> {
        self.make(programs,geometry,patina)?;
        Ok(&self.0.as_mut().unwrap().geometry)
    }

    fn get_patina(&mut self, programs: &ProgramStore<'c>, geometry: &GeometryProcessName, patina: &PatinaProcessName) -> anyhow::Result<&PatinaProcess> {
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

    fn holder(&mut self, patina: &PatinaProcessName) -> anyhow::Result<&mut SubLayerHolder<'c>> {
        Ok(match &patina{
            PatinaProcessName::Direct => &mut self.direct,
            PatinaProcessName::Spot(c) => self.spot.entry(c.clone()).or_insert_with(|| SubLayerHolder::new())
        })
    }

    fn get_process_mut(&mut self, programs: &ProgramStore<'c>, geometry: &GeometryProcessName, patina: &PatinaProcessName) -> anyhow::Result<&mut ProtoProcess<'c>> {
        self.holder(patina)?.get_process_mut(programs,geometry,patina)
    }

    fn get_geometry(&mut self, programs: &ProgramStore<'c>, geometry: &GeometryProcessName, patina: &PatinaProcessName) -> anyhow::Result<&GeometryProcess> {
        self.holder(patina)?.get_geometry(programs,geometry,patina)
    }

    fn get_patina(&mut self, programs: &ProgramStore<'c>, geometry: &GeometryProcessName, patina: &PatinaProcessName) -> anyhow::Result<&PatinaProcess> {
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

macro_rules! layer_geometry_accessor {
    ($func:ident,$geom_type:ty,$geom_name:ident) => {
        pub(crate) fn $func(&mut self, patina: &PatinaProcessName) -> anyhow::Result<$geom_type> {
            let geom = self.get_geometry(&GeometryProcessName::$geom_name,patina)?;
            match geom { GeometryProcess::$geom_name(x) => Ok(x.clone()), _ => bail!("inconsistent layer") }
        }
    };
}

macro_rules! layer_patina_accessor {
    ($func:ident,$patina_type:ty,$patina_name:ident) => {
        pub(crate) fn $func(&mut self, geometry: &GeometryProcessName) -> anyhow::Result<$patina_type> {
            let patina = self.get_patina(geometry,&PatinaProcessName::$patina_name)?;
            match patina { PatinaProcess::$patina_name(x) => Ok(x.clone()), _ => bail!("inconsistent layer") }
        }                
    };
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

    fn holder(&mut self, geometry: &GeometryProcessName) -> anyhow::Result<(&mut GeometrySubLayer<'c>,&'c ProgramStore<'c>)> {
        Ok(match geometry {
            GeometryProcessName::Pin => (&mut self.pin,&mut self.programs),
            GeometryProcessName::Fix => (&mut self.fix,&mut self.programs),
            GeometryProcessName::Tape => (&mut self.tape,&mut self.programs),
            GeometryProcessName::Page => (&mut self.page,&mut self.programs),
        })
    }

    pub(crate) fn get_process_mut(&mut self, geometry: &GeometryProcessName, patina: &PatinaProcessName) -> anyhow::Result<&mut ProtoProcess<'c>> {
        let (sub,compiler) = self.holder(geometry)?;
        sub.get_process_mut(compiler,geometry,patina)
    }

    fn get_geometry(&mut self, geometry: &GeometryProcessName, patina: &PatinaProcessName) -> anyhow::Result<&GeometryProcess> {
        let (sub,compiler) = self.holder(geometry)?;
       sub.get_geometry(compiler,geometry,patina)
    }

    fn get_patina(&mut self, geometry: &GeometryProcessName, patina: &PatinaProcessName) -> anyhow::Result<&PatinaProcess> {
        let (sub,compiler) = self.holder(geometry)?;
        sub.get_patina(compiler,geometry,patina)
    }

    pub(crate) fn make_campaign(&mut self, geometry: &GeometryProcessName, patina: &PatinaProcessName, count: usize, indexes: &[u16]) -> anyhow::Result<AccumulatorCampaign> {
        let process = self.get_process_mut(geometry,patina)?;
        Ok(process.get_accumulator().make_campaign(count,indexes)?)
    }

    layer_geometry_accessor!(get_pin,PinGeometry,Pin);
    layer_geometry_accessor!(get_fix,FixGeometry,Fix);
    layer_geometry_accessor!(get_page,PageGeometry,Page);
    layer_geometry_accessor!(get_tape,TapeGeometry,Tape);

    layer_patina_accessor!(get_direct,DirectColourDraw,Direct);

    pub(crate) fn get_spot(&mut self, geometry: &GeometryProcessName, colour: &DirectColour) -> anyhow::Result<SpotColourDraw> {
        let patina = self.get_patina(geometry,&PatinaProcessName::Spot(colour.clone()))?;
        match patina { PatinaProcess::Spot(x) => Ok(x.clone()), _ => bail!("inconsistent layer") }
    }
}
