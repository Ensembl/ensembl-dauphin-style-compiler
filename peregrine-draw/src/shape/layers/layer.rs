use std::collections::HashMap;
use super::super::core::pingeometry::PinGeometry;
use super::super::core::fixgeometry::FixGeometry;
use super::super::core::tapegeometry::TapeGeometry;
use super::super::core::pagegeometry::PageGeometry;
use super::super::core::wigglegeometry::WiggleGeometry;
use super::super::core::directcolourdraw::DirectColourDraw;
use super::super::core::spotcolourdraw::SpotColourDraw;
use super::super::core::texture::TextureDraw;
use crate::webgl::FlatId;
use crate::webgl::{ ProtoProcess, Process, ProcessStanzaElements, ProcessStanzaArray, DrawingFlats };
use super::geometry::{ GeometryProcess, GeometryProcessName };
use super::programstore::ProgramStore;
use super::patina::{ PatinaProcess, PatinaProcessName };
use peregrine_data::DirectColour;
use crate::util::message::Message;
use crate::webgl::global::WebGlGlobal;
use web_sys::{ WebGlRenderingContext };
use crate::webgl::GPUSpec;

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

struct SubLayer {
    process: ProtoProcess,
    geometry: GeometryProcess,
    patina: PatinaProcess
}

struct SubLayerHolder(Option<SubLayer>,f64);

impl SubLayerHolder {
    fn new(left: f64) -> SubLayerHolder { SubLayerHolder(None,left) }

    fn make(&mut self, context: &WebGlRenderingContext, gpuspec: &GPUSpec, programs: &ProgramStore, geometry: &GeometryProcessName, patina: &PatinaProcessName) -> Result<(),Message> {
        let program_store_entry = programs.get_program(context,gpuspec,geometry.get_program_name(),patina.get_program_name())?;
        let process = ProtoProcess::new(program_store_entry.program().clone(),self.1);
        let geometry = program_store_entry.get_geometry().make_geometry_process(patina)?;
        let patina = program_store_entry.get_patina().make_patina_process(patina)?;
        self.0 = Some(SubLayer { process, geometry, patina });
        Ok(())
    }

    fn get_process_mut(&mut self, context: &WebGlRenderingContext, gpuspec: &GPUSpec, programs: &ProgramStore, geometry: &GeometryProcessName, patina: &PatinaProcessName) -> Result<&mut ProtoProcess,Message> {
        if self.0.is_none() { self.make(context,gpuspec,programs,geometry,patina)?; }
        Ok(&mut self.0.as_mut().unwrap().process)
    }

    fn get_geometry(&mut self, context: &WebGlRenderingContext, gpuspec: &GPUSpec, programs: &ProgramStore, geometry: &GeometryProcessName, patina: &PatinaProcessName) -> Result<&GeometryProcess,Message> {
        if self.0.is_none() { self.make(context,gpuspec,programs,geometry,patina)?; }
        Ok(&self.0.as_mut().unwrap().geometry)
    }

    fn get_patina(&mut self, context: &WebGlRenderingContext, gpuspec: &GPUSpec,programs: &ProgramStore, geometry: &GeometryProcessName, patina: &PatinaProcessName) -> Result<&PatinaProcess,Message> {
        if self.0.is_none() { self.make(context,gpuspec,programs,geometry,patina)?; }
        Ok(&self.0.as_mut().unwrap().patina)
    }

    fn set_canvases(&mut self, canvases: &DrawingFlats) -> Result<(),Message> {
        canvases.add_process(&mut self.0.as_mut().unwrap().process)
    }

    fn build(self,gl: &mut WebGlGlobal) -> Result<Option<Process>,Message> {
        if let Some(sub) = self.0 {
            Ok(Some(sub.process.build(gl)?))
        } else {
            Ok(None)
        }
    }
}

struct GeometrySubLayer {
    direct: SubLayerHolder,
    spot: HashMap<DirectColour,SubLayerHolder>,
    texture: HashMap<FlatId,SubLayerHolder>,
    left: f64
}

impl GeometrySubLayer {
    fn new(left: f64) -> GeometrySubLayer {
        GeometrySubLayer {
            direct: SubLayerHolder::new(left),
            spot: HashMap::new(),
            texture: HashMap::new(),
            left
        }
    }

    fn holder(&mut self, patina: &PatinaProcessName) -> Result<&mut SubLayerHolder,Message> {
        let left = self.left;
        Ok(match &patina{
            PatinaProcessName::Direct => &mut self.direct,
            PatinaProcessName::Spot(c) => self.spot.entry(c.clone()).or_insert_with(|| SubLayerHolder::new(left)),
            PatinaProcessName::Texture(c) => self.texture.entry(c.clone()).or_insert_with(|| SubLayerHolder::new(left)),
        })
    }

    fn get_process_mut(&mut self, context: &WebGlRenderingContext, gpuspec: &GPUSpec, programs: &ProgramStore, geometry: &GeometryProcessName, patina: &PatinaProcessName) -> Result<&mut ProtoProcess,Message> {
        self.holder(patina)?.get_process_mut(context,gpuspec,programs,geometry,patina)
    }

    fn get_geometry(&mut self, context: &WebGlRenderingContext, gpuspec: &GPUSpec,programs: &ProgramStore, geometry: &GeometryProcessName, patina: &PatinaProcessName) -> Result<&GeometryProcess,Message> {
        self.holder(patina)?.get_geometry(context,gpuspec,programs,geometry,patina)
    }

    fn get_patina(&mut self, context: &WebGlRenderingContext, gpuspec: &GPUSpec,programs: &ProgramStore, geometry: &GeometryProcessName, patina: &PatinaProcessName) -> Result<&PatinaProcess,Message> {
        self.holder(patina)?.get_patina(context,gpuspec,programs,geometry,patina)
    }

    fn build(mut self, gl: &mut WebGlGlobal, processes: &mut Vec<Process>, canvases: &DrawingFlats) -> Result<(),Message> {
        if let Some(process) = self.direct.build(gl)? {
            processes.push(process);
        }
        for (_,sub) in self.spot.drain() {
            if let Some(process) = sub.build(gl)? {
                processes.push(process);
            }
        }
        for (_,mut sub) in self.texture.drain() {
            sub.set_canvases(canvases)?;
            if let Some(process) = sub.build(gl)? {
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
    wiggle: GeometrySubLayer,
    left: f64
}

macro_rules! layer_geometry_accessor {
    ($func:ident,$geom_type:ty,$geom_name:ident) => {
        pub(crate) fn $func(&mut self, context: &WebGlRenderingContext, gpuspec: &GPUSpec, patina: &PatinaProcessName) -> Result<$geom_type,Message> {
            let geom = self.get_geometry(context,gpuspec,&GeometryProcessName::$geom_name,patina)?;
            match geom { GeometryProcess::$geom_name(x) => Ok(x.clone()), _ => Err(Message::CodeInvariantFailed(format!("inconsistent layer A"))) }
        }
    };
}

macro_rules! layer_patina_accessor {
    ($func:ident,$patina_type:ty,$patina_name:ident) => {
        pub(crate) fn $func(&mut self, context: &WebGlRenderingContext, gpuspec: &GPUSpec, geometry: &GeometryProcessName) -> Result<$patina_type,Message> {
            let patina = self.get_patina(context,gpuspec,geometry,&PatinaProcessName::$patina_name)?;
            match patina { PatinaProcess::$patina_name(x) => Ok(x.clone()), _ =>  Err(Message::CodeInvariantFailed(format!("inconsistent layer B"))) }
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
            wiggle: GeometrySubLayer::new(left),
            left
        }
    }

    pub(crate) fn left(&self) -> f64 { self.left }

    fn holder(&mut self, geometry: &GeometryProcessName) -> Result<(&mut GeometrySubLayer,&ProgramStore),Message> {
        Ok(match geometry {
            GeometryProcessName::Pin => (&mut self.pin,&self.programs),
            GeometryProcessName::Fix => (&mut self.fix,&self.programs),
            GeometryProcessName::Tape => (&mut self.tape,&self.programs),
            GeometryProcessName::Page => (&mut self.page,&self.programs),
            GeometryProcessName::Wiggle => (&mut self.wiggle,&self.programs)
        })
    }

    pub(crate) fn get_process_mut(&mut self, context: &WebGlRenderingContext, gpuspec: &GPUSpec, geometry: &GeometryProcessName, patina: &PatinaProcessName) -> Result<&mut ProtoProcess,Message> {
        let (sub,compiler) = self.holder(geometry)?;
        sub.get_process_mut(context,gpuspec,compiler,geometry,patina)
    }

    fn get_geometry(&mut self, context: &WebGlRenderingContext, gpuspec: &GPUSpec, geometry: &GeometryProcessName, patina: &PatinaProcessName) -> Result<&GeometryProcess,Message> {
        let (sub,compiler) = self.holder(geometry)?;
       sub.get_geometry(context,gpuspec,compiler,geometry,patina)
    }

    fn get_patina(&mut self, context: &WebGlRenderingContext, gpuspec: &GPUSpec, geometry: &GeometryProcessName, patina: &PatinaProcessName) -> Result<&PatinaProcess,Message> {
        let (sub,compiler) = self.holder(geometry)?;
        sub.get_patina(context,gpuspec,compiler,geometry,patina)
    }

    pub(crate) fn make_elements(&mut self, context: &WebGlRenderingContext, gpuspec: &GPUSpec,geometry: &GeometryProcessName, patina: &PatinaProcessName, count: usize, indexes: &[u16]) -> Result<ProcessStanzaElements,Message> {
        let process = self.get_process_mut(context,gpuspec,geometry,patina)?;
        Ok(process.get_stanza_builder().make_elements(count,indexes)?)
    }

    pub(crate) fn make_array(&mut self, context: &WebGlRenderingContext, gpuspec: &GPUSpec, geometry: &GeometryProcessName, patina: &PatinaProcessName, count: usize) ->Result<ProcessStanzaArray,Message> {
        let process = self.get_process_mut(context,gpuspec,geometry,patina)?;
        Ok(process.get_stanza_builder().make_array(count)?)
    }

    layer_geometry_accessor!(get_pin,PinGeometry,Pin);
    layer_geometry_accessor!(get_fix,FixGeometry,Fix);
    layer_geometry_accessor!(get_page,PageGeometry,Page);
    layer_geometry_accessor!(get_tape,TapeGeometry,Tape);
    layer_geometry_accessor!(get_wiggle,WiggleGeometry,Wiggle);

    layer_patina_accessor!(get_direct,DirectColourDraw,Direct);

    pub(crate) fn get_spot(&mut self, context: &WebGlRenderingContext, gpuspec: &GPUSpec, geometry: &GeometryProcessName, colour: &DirectColour) -> Result<SpotColourDraw,Message> {
        let patina = self.get_patina(context,gpuspec,geometry,&PatinaProcessName::Spot(colour.clone()))?;
        match patina { PatinaProcess::Spot(x) => Ok(x.clone()), _ => Err(Message::CodeInvariantFailed(format!("inconsistent layer C"))) }
    }

    pub(crate) fn get_texture(&mut self, context: &WebGlRenderingContext, gpuspec: &GPUSpec, geometry: &GeometryProcessName, element_id: &FlatId) -> Result<TextureDraw,Message> {
        let patina = self.get_patina(context,gpuspec,geometry,&PatinaProcessName::Texture(element_id.clone()))?;
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
