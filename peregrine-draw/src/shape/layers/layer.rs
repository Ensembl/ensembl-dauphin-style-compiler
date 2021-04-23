use std::collections::HashMap;
use super::super::core::fixgeometry::FixProgram;
use super::super::core::tapegeometry::TapeProgram;
use super::super::core::pagegeometry::PageProgram;
use super::super::core::pingeometry::PinProgram;
use super::super::core::wigglegeometry::WiggleProgram;
use super::super::core::directcolourdraw::DirectColourDraw;
use super::super::core::spotcolourdraw::SpotColourDraw;
use super::super::core::texture::TextureDraw;
use crate::webgl::FlatId;
use crate::webgl::{ ProcessBuilder, Process, DrawingFlats };
use super::geometry::{ GeometryProgramName, GeometryProgram };
use super::programstore::ProgramStore;
use super::patina::{ PatinaProcess, PatinaProcessName };
use peregrine_data::DirectColour;
use crate::util::message::Message;
use crate::webgl::global::WebGlGlobal;
use super::shapeprogram::ShapeProgram;

/* 
TODO ensure + index
TODO y split bug
TODO y from bottom
TODO layers from core
TODO ordered layers
TODO remove datum option from stretchtangles
TODO return shapes from core without cloning (drain)
TODO uniforms set only on change
TODO global destroy
TODO keep program when same program
TODO initial clear
TODO wiggle width
TODO hollowwidth
TODO intersection cache
*/

struct GeometrySubLayer {
    direct: Option<ShapeProgram>,
    spot: HashMap<DirectColour,ShapeProgram>,
    texture: HashMap<FlatId,ShapeProgram>,
    geometry_program_name: GeometryProgramName,
    left: f64
}

impl GeometrySubLayer {
    fn new(geometry_program_name: &GeometryProgramName, left: f64) -> Result<GeometrySubLayer,Message> {
        Ok(GeometrySubLayer {
            direct: None,
            spot: HashMap::new(),
            texture: HashMap::new(),
            geometry_program_name: geometry_program_name.clone(),
            left
        })
    }

    fn holder(&mut self, programs: &ProgramStore, patina: &PatinaProcessName) -> Result<&mut ShapeProgram,Message> {
        let geometry = self.geometry_program_name.clone();
        Ok(match &patina {
            PatinaProcessName::Direct => {
                if self.direct.is_none() {
                    self.direct = Some(ShapeProgram::new(programs,&geometry,&patina)?);
                }
                self.direct.as_mut().unwrap()
            }
            PatinaProcessName::Spot(c) => {
                if !self.spot.contains_key(c) {
                    self.spot.insert(c.clone(), ShapeProgram::new(programs,&geometry,patina)?);
                }
                self.spot.get_mut(c).unwrap()
            },
            PatinaProcessName::Texture(c) => {
                if !self.texture.contains_key(c) {
                    self.texture.insert(c.clone(), ShapeProgram::new(programs,&geometry,patina)?);
                }
                self.texture.get_mut(c).unwrap()
            }
        })
    }

    fn get_process_mut(&mut self, programs: &ProgramStore, patina: &PatinaProcessName) -> Result<&mut ProcessBuilder,Message> {
        Ok(self.holder(programs,patina)?.get_process_mut())
    }

    fn get_geometry(&mut self, programs: &ProgramStore, patina: &PatinaProcessName) -> Result<&GeometryProgram,Message> {
        Ok(self.holder(programs,patina)?.get_geometry())
    }

    fn get_patina(&mut self, programs: &ProgramStore, patina: &PatinaProcessName) -> Result<&PatinaProcess,Message> {
        Ok(self.holder(programs,patina)?.get_patina())
    }

    fn build(mut self, gl: &mut WebGlGlobal, processes: &mut Vec<Process>, canvases: &DrawingFlats) -> Result<(),Message> {
        if let Some(direct) = self.direct {
            processes.push(direct.into_process().build(gl,self.left)?);
        }
        for (_,sub) in self.spot.drain() {
            processes.push(sub.into_process().build(gl,self.left)?);
        }
        for (_,mut sub) in self.texture.drain() {
            canvases.add_process(sub.get_process_mut())?;
            processes.push(sub.into_process().build(gl,self.left)?);
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
    wiggle: GeometrySubLayer,
    left: f64
}

macro_rules! layer_geometry_accessor {
    ($func:ident,$geom_type:ty,$geom_name:ident) => {
        pub(crate) fn $func(&mut self, patina: &PatinaProcessName) -> Result<$geom_type,Message> {
            let geom = self.get_geometry(&GeometryProgramName::$geom_name,patina)?;
            match geom { GeometryProgram::$geom_name(x) => Ok(x.clone()), _ => Err(Message::CodeInvariantFailed(format!("inconsistent layer A"))) }
        }
    };
}

macro_rules! layer_patina_accessor {
    ($func:ident,$patina_type:ty,$patina_name:ident) => {
        pub(crate) fn $func(&mut self, geometry: &GeometryProgramName) -> Result<$patina_type,Message> {
            let patina = self.get_patina(geometry,&PatinaProcessName::$patina_name)?;
            match patina { PatinaProcess::$patina_name(x) => Ok(x.clone()), _ =>  Err(Message::CodeInvariantFailed(format!("inconsistent layer B"))) }
        }                
    };
}

impl Layer {
    pub fn new(programs: &ProgramStore, left: f64) -> Result<Layer,Message> {
        Ok(Layer {
            programs: programs.clone(),
            pin: GeometrySubLayer::new(&GeometryProgramName::Pin,left)?,
            fix: GeometrySubLayer::new(&GeometryProgramName::Fix,left)?,
            tape: GeometrySubLayer::new(&GeometryProgramName::Tape,left)?,
            page: GeometrySubLayer::new(&GeometryProgramName::Page,left)?,
            wiggle: GeometrySubLayer::new(&GeometryProgramName::Wiggle,left)?,
            left
        })
    }

    pub(crate) fn left(&self) -> f64 { self.left }

    fn holder(&mut self, geometry: &GeometryProgramName) -> Result<(&mut GeometrySubLayer,&ProgramStore),Message> {
        Ok(match geometry {
            GeometryProgramName::Pin => (&mut self.pin,&self.programs),
            GeometryProgramName::Fix => (&mut self.fix,&self.programs),
            GeometryProgramName::Tape => (&mut self.tape,&self.programs),
            GeometryProgramName::Page => (&mut self.page,&self.programs),
            GeometryProgramName::Wiggle => (&mut self.wiggle,&self.programs)
        })
    }

    pub(crate) fn get_process_mut(&mut self,  geometry: &GeometryProgramName, patina: &PatinaProcessName) -> Result<&mut ProcessBuilder,Message> {
        let (sub,compiler) = self.holder(geometry)?;
        sub.get_process_mut(compiler,patina)
    }

    fn get_geometry(&mut self, geometry: &GeometryProgramName, patina: &PatinaProcessName) -> Result<&GeometryProgram,Message> {
        let (sub,compiler) = self.holder(geometry)?;
       sub.get_geometry(compiler,patina)
    }

    fn get_patina(&mut self, geometry: &GeometryProgramName, patina: &PatinaProcessName) -> Result<&PatinaProcess,Message> {
        let (sub,compiler) = self.holder(geometry)?;
        sub.get_patina(compiler,patina)
    }

    layer_geometry_accessor!(get_pin,PinProgram,Pin);
    layer_geometry_accessor!(get_fix,FixProgram,Fix);
    layer_geometry_accessor!(get_page,PageProgram,Page);
    layer_geometry_accessor!(get_tape,TapeProgram,Tape);
    layer_geometry_accessor!(get_wiggle,WiggleProgram,Wiggle);

    layer_patina_accessor!(get_direct,DirectColourDraw,Direct);

    pub(crate) fn get_spot(&mut self, geometry: &GeometryProgramName, colour: &DirectColour) -> Result<SpotColourDraw,Message> {
        let patina = self.get_patina(geometry,&PatinaProcessName::Spot(colour.clone()))?;
        match patina { PatinaProcess::Spot(x) => Ok(x.clone()), _ => Err(Message::CodeInvariantFailed(format!("inconsistent layer C"))) }
    }

    pub(crate) fn get_texture(&mut self, geometry: &GeometryProgramName, element_id: &FlatId) -> Result<TextureDraw,Message> {
        let patina = self.get_patina(geometry,&PatinaProcessName::Texture(element_id.clone()))?;
        match patina { PatinaProcess::Texture(x) => Ok(x.clone()), _ => Err(Message::CodeInvariantFailed(format!("inconsistent layer D"))) }
    }

    pub(super) fn build(self, gl: &mut WebGlGlobal, process: &mut Vec<Process>, canvases: &DrawingFlats) -> Result<(),Message> {
        self.pin.build(gl,process,canvases)?;
        self.tape.build(gl,process,canvases)?;
        self.page.build(gl,process,canvases)?;
        self.fix.build(gl,process,canvases)?;
        Ok(())
    }
}
