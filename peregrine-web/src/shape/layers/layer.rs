use anyhow::bail;
use std::collections::HashMap;
use super::super::core::pingeometry::PinGeometry;
use super::super::core::fixgeometry::FixGeometry;
use super::super::core::tapegeometry::TapeGeometry;
use super::super::core::pagegeometry::PageGeometry;
use super::super::core::directcolourdraw::DirectColourDraw;
use super::super::core::spotcolourdraw::SpotColourDraw;
use crate::webgl::{ ProtoProcess, Process, AccumulatorCampaign };
use super::geometry::{ GeometryProcess, GeometryProcessName };
use super::programstore::ProgramStore;
use super::patina::{ PatinaProcess, PatinaProcessName };
use peregrine_core::DirectColour;

/* 
TODO Wiggles
TODO ensure + index
TODO y split bug
TODO y from bottom
TODO layers from core
TODO ordered layers
TODO remove datum option from stretchtangles
TODO return shapes from core without cloning (drain)
TODO uniforms set only on change
TODO global destroy
*/

struct SubLayer {
    process: ProtoProcess,
    geometry: GeometryProcess,
    patina: PatinaProcess
}

struct SubLayerHolder(Option<SubLayer>,f64);

impl SubLayerHolder {
    fn new(left: f64) -> SubLayerHolder { SubLayerHolder(None,left) }

    fn make(&mut self, programs: &ProgramStore, geometry: &GeometryProcessName, patina: &PatinaProcessName) -> anyhow::Result<()> {
        let program_store_entry = programs.get_program(geometry.get_program_name(),patina.get_program_name())?;
        let process = ProtoProcess::new(program_store_entry.program().clone(),self.1);
        let geometry = program_store_entry.get_geometry().make_geometry_process(&process,patina)?;
        let patina = program_store_entry.get_patina().make_patina_process(&process,patina)?;
        self.0 = Some(SubLayer { process, geometry, patina });
        Ok(())
    }

    fn get_process_mut(&mut self, programs: &ProgramStore, geometry: &GeometryProcessName, patina: &PatinaProcessName) -> anyhow::Result<&mut ProtoProcess> {
        self.make(programs,geometry,patina)?;
        Ok(&mut self.0.as_mut().unwrap().process)
    }

    fn get_geometry(&mut self, programs: &ProgramStore, geometry: &GeometryProcessName, patina: &PatinaProcessName) -> anyhow::Result<&GeometryProcess> {
        self.make(programs,geometry,patina)?;
        Ok(&self.0.as_mut().unwrap().geometry)
    }

    fn get_patina(&mut self, programs: &ProgramStore, geometry: &GeometryProcessName, patina: &PatinaProcessName) -> anyhow::Result<&PatinaProcess> {
        self.make(programs,geometry,patina)?;
        Ok(&self.0.as_mut().unwrap().patina)
    }

    fn build(self) -> anyhow::Result<Option<Process>> {
        if let Some(sub) = self.0 {
            Ok(Some(sub.process.build()?))
        } else {
            Ok(None)
        }
    }
}

struct GeometrySubLayer {
    direct: SubLayerHolder,
    spot: HashMap<DirectColour,SubLayerHolder>,
    left: f64
}

impl GeometrySubLayer {
    fn new(left: f64) -> GeometrySubLayer {
        GeometrySubLayer {
            direct: SubLayerHolder::new(left),
            spot: HashMap::new(),
            left
        }
    }

    fn holder(&mut self, patina: &PatinaProcessName) -> anyhow::Result<&mut SubLayerHolder> {
        let left = self.left;
        Ok(match &patina{
            PatinaProcessName::Direct => &mut self.direct,
            PatinaProcessName::Spot(c) => self.spot.entry(c.clone()).or_insert_with(|| SubLayerHolder::new(left))
        })
    }

    fn get_process_mut(&mut self, programs: &ProgramStore, geometry: &GeometryProcessName, patina: &PatinaProcessName) -> anyhow::Result<&mut ProtoProcess> {
        self.holder(patina)?.get_process_mut(programs,geometry,patina)
    }

    fn get_geometry(&mut self, programs: &ProgramStore, geometry: &GeometryProcessName, patina: &PatinaProcessName) -> anyhow::Result<&GeometryProcess> {
        self.holder(patina)?.get_geometry(programs,geometry,patina)
    }

    fn get_patina(&mut self, programs: &ProgramStore, geometry: &GeometryProcessName, patina: &PatinaProcessName) -> anyhow::Result<&PatinaProcess> {
        self.holder(patina)?.get_patina(programs,geometry,patina)
    }

    fn build(mut self, processes: &mut Vec<Process>) -> anyhow::Result<()> {
        if let Some(process) = self.direct.build()? {
            processes.push(process);
        }
        for (_,sub) in self.spot.drain() {
            if let Some(process) = sub.build()? {
                processes.push(process);
            }
        }
        Ok(())
    }
}

pub(crate) struct Layer {
    programs: ProgramStore,
    pin: GeometrySubLayer,
    fix: GeometrySubLayer,
    tape: GeometrySubLayer,
    page: GeometrySubLayer,
    left: f64
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

impl Layer {
    pub fn new(programs: &ProgramStore, left: f64) -> Layer {
        Layer {
            programs: programs.clone(),
            pin: GeometrySubLayer::new(left),
            fix: GeometrySubLayer::new(left),
            tape: GeometrySubLayer::new(left),
            page: GeometrySubLayer::new(left),
            left
        }
    }

    pub(crate) fn left(&self) -> f64 { self.left }

    fn holder(&mut self, geometry: &GeometryProcessName) -> anyhow::Result<(&mut GeometrySubLayer,&ProgramStore)> {
        Ok(match geometry {
            GeometryProcessName::Pin => (&mut self.pin,&self.programs),
            GeometryProcessName::Fix => (&mut self.fix,&self.programs),
            GeometryProcessName::Tape => (&mut self.tape,&self.programs),
            GeometryProcessName::Page => (&mut self.page,&self.programs),
        })
    }

    pub(crate) fn get_process_mut(&mut self, geometry: &GeometryProcessName, patina: &PatinaProcessName) -> anyhow::Result<&mut ProtoProcess> {
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

    pub(super) fn build(self, process: &mut Vec<Process>) -> anyhow::Result<()> {
        self.pin.build(process)?;
        self.tape.build(process)?;
        self.page.build(process)?;
        self.fix.build(process)?;
        Ok(())
    }
}
