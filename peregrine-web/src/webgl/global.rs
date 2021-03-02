use crate::shape::layers::programstore::ProgramStore;
use crate::webgl::{ FlatStore, TextureBindery,TextureStore };
use web_sys::Document;
use crate::webgl::util::handle_context_errors;

#[cfg(blackbox)]
use crate::integration::pgblackbox::{ pgblackbox_setup };
pub use url::Url;
pub use web_sys::{ console, WebGlRenderingContext };

pub struct WebGlGlobal {
    program_store: ProgramStore,
    context: WebGlRenderingContext,
    canvas_store: FlatStore,
    bindery: TextureBindery,
    texture_store: TextureStore,
    document: Document
}

impl WebGlGlobal {
    pub(crate) fn new(document: &Document, context: &WebGlRenderingContext) -> anyhow::Result<WebGlGlobal> {
        let program_store = ProgramStore::new(&context)?;
        let canvas_store = FlatStore::new();
        let bindery = TextureBindery::new(program_store.gpu_spec());
        Ok(WebGlGlobal {
            program_store, 
            canvas_store, 
            bindery,
            texture_store: TextureStore::new(),
            context: context.clone(),
            document: document.clone()
        })
    }

    pub(crate) fn document(&self) -> &Document { &self.document }
    pub(crate) fn program_store(&self) -> &ProgramStore { &self.program_store }
    pub(crate) fn context(&self) -> &WebGlRenderingContext { &self.context }
    pub(crate) fn flat_store(&self) -> &FlatStore { &self.canvas_store }
    pub(crate) fn canvas_store_mut(&mut self) -> &mut FlatStore { &mut self.canvas_store }
    pub(crate) fn bindery(&mut self) -> &mut TextureBindery { &mut self.bindery }
    pub(crate) fn texture_store(&mut self) -> &mut TextureStore { &mut self.texture_store }

    pub(crate) fn handle_context_errors(&mut self) -> anyhow::Result<()> {
        handle_context_errors(&self.context)?;
        Ok(())
    }
}