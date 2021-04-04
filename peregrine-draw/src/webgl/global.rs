use crate::shape::layers::programstore::ProgramStore;
use crate::webgl::{ FlatStore, TextureBindery,TextureStore };
use web_sys::Document;
use crate::webgl::util::handle_context_errors;
pub use url::Url;
pub use web_sys::{ console, WebGlRenderingContext };
use crate::PeregrineDom;
use crate::util::message::Message;
use wasm_bindgen::JsCast;

pub struct WebGlGlobal {
    program_store: ProgramStore,
    context: WebGlRenderingContext,
    canvas_store: FlatStore,
    bindery: TextureBindery,
    texture_store: TextureStore,
    document: Document,
    canvas_size: Option<(u32,u32)>
}

impl WebGlGlobal {
    pub(crate) fn new(dom: &PeregrineDom) -> Result<WebGlGlobal,Message> {
        let context = dom.canvas()
            .get_context("webgl").map_err(|_| Message::WebGLFailure(format!("cannot get webgl context")))?
            .unwrap()
            .dyn_into::<WebGlRenderingContext>().map_err(|_| Message::WebGLFailure(format!("cannot get webgl context")))?;

        let program_store = ProgramStore::new(&context)?;
        let canvas_store = FlatStore::new();
        let bindery = TextureBindery::new(program_store.gpu_spec());
        Ok(WebGlGlobal {
            program_store, 
            canvas_store, 
            bindery,
            texture_store: TextureStore::new(),
            context: context.clone(),
            document: dom.document().clone(),
            canvas_size: None
        })
    }

    pub(crate) fn document(&self) -> &Document { &self.document }
    pub(crate) fn program_store(&self) -> &ProgramStore { &self.program_store }
    pub(crate) fn context(&self) -> &WebGlRenderingContext { &self.context }
    pub(crate) fn flat_store(&self) -> &FlatStore { &self.canvas_store }
    pub(crate) fn canvas_store_mut(&mut self) -> &mut FlatStore { &mut self.canvas_store }
    pub(crate) fn bindery(&mut self) -> &mut TextureBindery { &mut self.bindery }
    pub(crate) fn texture_store(&mut self) -> &mut TextureStore { &mut self.texture_store }
    pub(crate) fn canvas_size(&mut self) -> &mut Option<(u32,u32)> { &mut self.canvas_size }

    pub(crate) fn handle_context_errors(&mut self) -> Result<(),Message> {
        handle_context_errors(&self.context)?;
        Ok(())
    }
}
