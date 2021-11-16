use crate::{run::{ PgPeregrineConfig, PgConfigKey }, shape::layers::programstore::ProgramStore};
use crate::webgl::{ FlatStore, TextureBindery };
use js_sys::Float32Array;
use web_sys::Document;
pub use url::Url;
pub use web_sys::{ console, WebGlRenderingContext };
use crate::PeregrineDom;
use crate::util::message::Message;
use wasm_bindgen::JsCast;
use super::GPUSpec;

pub struct WebGlGlobal {
    program_store: ProgramStore,
    context: WebGlRenderingContext,
    canvas_store: FlatStore,
    bindery: TextureBindery,
    document: Document,
    canvas_size: Option<(u32,u32)>,
    gpuspec: GPUSpec,
    aux_array: Float32Array
}

pub(crate) struct WebGlGlobalRefs<'a> {
    pub program_store: &'a ProgramStore,
    pub context: &'a WebGlRenderingContext,
    pub flat_store: &'a mut FlatStore,
    pub bindery: &'a mut TextureBindery,
    pub document: &'a Document,
    pub canvas_size: &'a mut Option<(u32,u32)>,
    pub gpuspec: &'a GPUSpec,
    pub aux_array: &'a Float32Array
}

impl WebGlGlobal {
    pub(crate) fn new(dom: &PeregrineDom, config: &PgPeregrineConfig) -> Result<WebGlGlobal,Message> {
        let context = dom.canvas()
            .get_context("webgl").map_err(|_| Message::WebGLFailure(format!("cannot get webgl context")))?
            .unwrap()
            .dyn_into::<WebGlRenderingContext>().map_err(|_| Message::WebGLFailure(format!("cannot get webgl context")))?;
        let gpuspec = GPUSpec::new(&context)?;
        let program_store = ProgramStore::new()?;
        let canvas_store = FlatStore::new();
        let bindery = TextureBindery::new(&gpuspec);
        Ok(WebGlGlobal {
            program_store, 
            canvas_store, 
            bindery,
            context: context.clone(),
            document: dom.document().clone(),
            canvas_size: None,
            gpuspec,
            aux_array: Float32Array::new_with_length(config.get_size(&PgConfigKey::AuxBufferSize)? as u32)
        })
    }

    pub(crate) fn refs<'a>(&'a mut self) -> WebGlGlobalRefs<'a> {
        WebGlGlobalRefs {
            program_store: &self.program_store,
            context: &self.context,
            flat_store: &mut self.canvas_store,
            bindery: &mut self.bindery,
            document: &self.document,
            canvas_size: &mut self.canvas_size,
            gpuspec: &self.gpuspec,
            aux_array: &self.aux_array       
        }
    }
}
