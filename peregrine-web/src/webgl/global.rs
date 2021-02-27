use anyhow::{ self, Context, anyhow as err };
use crate::shape::layers::programstore::ProgramStore;
use crate::webgl::canvas::flatstore::FlatStore;
use web_sys::Document;

#[cfg(blackbox)]
use crate::integration::pgblackbox::{ pgblackbox_setup };
pub use url::Url;
pub use web_sys::{ console, WebGlRenderingContext };

// XXX not pub
pub struct WebGlGlobal {
    program_store: ProgramStore,
    context: WebGlRenderingContext,
    canvas_store: FlatStore,
    document: Document
}

impl WebGlGlobal {
    pub(crate) fn new(document: &Document, context: &WebGlRenderingContext) -> anyhow::Result<WebGlGlobal> {
        let program_store = ProgramStore::new(&context)?;
        let canvas_store = FlatStore::new();
        Ok(WebGlGlobal { program_store, canvas_store, context: context.clone(), document: document.clone() })
    }

    pub(crate) fn document(&self) -> &Document { &self.document }
    pub(crate) fn program_store(&self) -> &ProgramStore { &self.program_store }
    pub(crate) fn context(&self) -> &WebGlRenderingContext { &self.context }
    pub(crate) fn canvas_store(&self) -> &FlatStore { &self.canvas_store }
    pub(crate) fn canvas_store_mut(&mut self) -> &mut FlatStore { &mut self.canvas_store }
}