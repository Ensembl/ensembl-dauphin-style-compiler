use crate::{run::{ PgPeregrineConfig }, shape::layers::programstore::ProgramStore, util::fonts::Fonts, PgCommanderWeb, domcss::dom::PeregrineDom};
use crate::webgl::{ ScratchCanvasAllocator, TextureBindery };
pub use url::Url;
pub use web_sys::{ console, WebGlRenderingContext };
use crate::util::message::Message;
use wasm_bindgen::JsCast;
use super::{GPUSpec, glbufferstore::GLBufferStore, canvas::{canvassource::CanvasSource, imagecache::{ImageCache}}};

pub struct WebGlGlobal {
    program_store: ProgramStore,
    context: WebGlRenderingContext,
    canvas_source: CanvasSource,
    scratch_canvases: ScratchCanvasAllocator,
    image_cache: ImageCache,
    bindery: TextureBindery,
    canvas_size: Option<(u32,u32)>,
    gpuspec: GPUSpec,
    fonts: Fonts,
    dpr: f32,
    buffer_store: GLBufferStore
}

pub(crate) struct WebGlGlobalRefs<'a> {
    pub program_store: &'a ProgramStore,
    pub context: &'a WebGlRenderingContext,
    pub image_cache: &'a ImageCache,
    pub scratch_canvases: &'a mut ScratchCanvasAllocator,
    pub canvas_source: &'a mut CanvasSource,
    pub bindery: &'a mut TextureBindery,
    pub canvas_size: &'a mut Option<(u32,u32)>,
    pub gpuspec: &'a GPUSpec,
    pub fonts: &'a Fonts,
    pub buffer_store: &'a GLBufferStore
}

impl WebGlGlobal {
    pub(crate) fn new(commander: &PgCommanderWeb, dom: &PeregrineDom, config: &PgPeregrineConfig) -> Result<WebGlGlobal,Message> {
        let context = dom.canvas()
            .get_context("webgl").map_err(|_| Message::WebGLFailure(format!("cannot get webgl context")))?
            .unwrap()
            .dyn_into::<WebGlRenderingContext>().map_err(|_| Message::WebGLFailure(format!("cannot get webgl context")))?;
        let image_cache = ImageCache::new();
        let gpuspec = GPUSpec::new(&context)?;
        let program_store = ProgramStore::new(commander)?;
        let fonts = Fonts::new()?;
        let canvas_source = CanvasSource::new(dom.document(),dom.device_pixel_ratio());
        let scratch_canvases = ScratchCanvasAllocator::new(&canvas_source);
        let bindery = TextureBindery::new(&gpuspec);
        Ok(WebGlGlobal {
            program_store, 
            scratch_canvases, 
            canvas_source,
            image_cache,
            bindery,
            context: context.clone(),
            canvas_size: None,
            gpuspec,
            fonts,
            dpr: dom.device_pixel_ratio(),
            buffer_store: GLBufferStore::new(&context)
        })
    }

    pub fn device_pixel_ratio(&self) -> f32 { self.dpr }
    pub(crate) fn gpu_spec(&self) -> &GPUSpec { &self.gpuspec }
    pub fn canvas_source(&self) -> &CanvasSource { &self.canvas_source }

    pub(crate) fn refs<'a>(&'a mut self) -> WebGlGlobalRefs<'a> {
        WebGlGlobalRefs {
            program_store: &self.program_store,
            context: &self.context,
            image_cache: &self.image_cache,
            scratch_canvases: &mut self.scratch_canvases,
            bindery: &mut self.bindery,
            canvas_source: &mut self.canvas_source,
            canvas_size: &mut self.canvas_size,
            gpuspec: &self.gpuspec,
            fonts: &self.fonts,
            buffer_store: &self.buffer_store 
        }
    }
}
