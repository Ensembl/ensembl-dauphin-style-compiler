use crate::{run::{ PgPeregrineConfig, PgConfigKey }, shape::layers::programstore::ProgramStore};
use crate::webgl::{ FlatStore, TextureBindery };
use js_sys::Float32Array;
use web_sys::Document;
use crate::webgl::util::handle_context_errors;
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

    pub(crate) fn document(&self) -> &Document { &self.document }
    pub(crate) fn program_store(&self) -> &ProgramStore { &self.program_store }
    pub(crate) fn context(&self) -> &WebGlRenderingContext { &self.context }
    pub(crate) fn aux_array(&self) -> &Float32Array {&self.aux_array }
    pub(crate) fn flat_store(&self) -> &FlatStore { &self.canvas_store }
    pub(crate) fn flat_store_mut(&mut self) -> &mut FlatStore { &mut self.canvas_store }
    pub(crate) fn bindery(&self) -> &TextureBindery { &self.bindery }
    pub(crate) fn bindery_mut(&mut self) -> &mut TextureBindery { &mut self.bindery }
    pub(crate) fn canvas_size(&mut self) -> &mut Option<(u32,u32)> { &mut self.canvas_size }
    pub(crate) fn gpuspec(&self) -> &GPUSpec { &self.gpuspec }

    pub(crate) fn handle_context_errors(&mut self) -> Result<(),Message> {
        handle_context_errors(&self.context)?;
        Ok(())
    }
}
