use anyhow::{ bail, anyhow as err };
use std::collections::VecDeque;
use super::flatstore::FlatId;
use crate::util::keyed::KeyedData;
use crate::webgl::GPUSpec;
use crate::webgl::canvas::weave::CanvasWeave;
use crate::webgl::canvas::flatstore::FlatStore;
use web_sys::WebGlRenderingContext;
use web_sys::WebGlTexture;
use crate::webgl::util::handle_context_errors;
use crate::webgl::global::WebGlGlobal;

fn apply_weave(context: &WebGlRenderingContext,weave: &CanvasWeave) -> anyhow::Result<()> {
    let (minf,magf,wraps,wrapt) = match weave {
        CanvasWeave::Crisp =>
            (WebGlRenderingContext::NEAREST,WebGlRenderingContext::NEAREST,
                WebGlRenderingContext::CLAMP_TO_EDGE,WebGlRenderingContext::CLAMP_TO_EDGE),
        CanvasWeave::Fuzzy =>
            (WebGlRenderingContext::LINEAR,WebGlRenderingContext::LINEAR,
                WebGlRenderingContext::REPEAT,WebGlRenderingContext::REPEAT)
    };
    context.tex_parameteri(WebGlRenderingContext::TEXTURE_2D,
                        WebGlRenderingContext::TEXTURE_MIN_FILTER,
                        minf as i32);
    context.tex_parameteri(WebGlRenderingContext::TEXTURE_2D,
                        WebGlRenderingContext::TEXTURE_MAG_FILTER,
                        magf as i32);
    context.tex_parameteri(WebGlRenderingContext::TEXTURE_2D,
                        WebGlRenderingContext::TEXTURE_WRAP_S,
                        wraps as i32);
    context.tex_parameteri(WebGlRenderingContext::TEXTURE_2D,
                        WebGlRenderingContext::TEXTURE_WRAP_T,
                        wrapt as i32);
    handle_context_errors(context)?;
    Ok(())
}

fn create_texture(context: &WebGlRenderingContext,canvas_store: &FlatStore, our_data: &FlatId) -> anyhow::Result<WebGlTexture> {
    let canvas = canvas_store.get(our_data)?;
    let texture = context.create_texture().ok_or_else(|| err!("cannot create texture"))?;
    handle_context_errors(context)?;
    context.bind_texture(WebGlRenderingContext::TEXTURE_2D,Some(&texture));
    handle_context_errors(context)?;
    context.tex_image_2d_with_u32_and_u32_and_canvas( // wow
        WebGlRenderingContext::TEXTURE_2D,0,WebGlRenderingContext::RGBA as i32,WebGlRenderingContext::RGBA,
        WebGlRenderingContext::UNSIGNED_BYTE,canvas.element()?
    ).map_err(|e| err!("cannot bind texture: {:?}",&e.as_string()))?;
    handle_context_errors(context)?;
    apply_weave(context,canvas.weave())?;
    Ok(texture)
}

pub struct TextureStore(KeyedData<FlatId,Option<WebGlTexture>>);

impl TextureStore {
    pub fn new() -> TextureStore {
        TextureStore(KeyedData::new())
    }

    pub fn add(&mut self, id: &FlatId, texture: WebGlTexture) {
        self.0.insert(id,texture);
    }

    pub fn remove(&mut self, id: &FlatId) -> anyhow::Result<WebGlTexture> {
        self.0.remove(id).ok_or_else(|| err!("no such texture"))
    }
}

struct Binding {
    position: usize,
    gl_index: u32
}

pub struct Rebind {
    old_texture: Option<FlatId>,
    new_texture: Option<FlatId>,
    new_index: u32
}

impl Rebind {
    fn cached(index: u32) -> Rebind {
        Rebind { old_texture: None, new_texture: None, new_index: index }
    }

    fn remove(flat_id: FlatId) -> Rebind {
        Rebind { old_texture: Some(flat_id), new_texture: None, new_index: 0}
    }

    pub(crate) fn apply(&self, gl: &mut WebGlGlobal) -> anyhow::Result<()> {
        if let Some(old_id) = &self.old_texture {
            let old_flat = gl.texture_store().remove(old_id)?;
            gl.context().delete_texture(Some(&old_flat));
            gl.handle_context_errors()?;
        }
        if let Some(new_id) = &self.new_texture {
            let texture = create_texture(gl.context(),gl.flat_store(),new_id)?;
            gl.context().active_texture(WebGlRenderingContext::TEXTURE0 + self.new_index);
            gl.handle_context_errors()?;
            gl.context().bind_texture(WebGlRenderingContext::TEXTURE_2D,Some(&texture));
            gl.handle_context_errors()?;
            gl.texture_store().add(new_id,texture);
        }
        Ok(())
    }
}

pub struct TextureBindery {
    flat_to_binding: KeyedData<FlatId,Option<Binding>>,
    position_to_flat: Vec<Option<FlatId>>,
    lru: VecDeque<usize>,
    in_use: Vec<usize>,
    next_gl_index: u32
}

impl TextureBindery {
    pub(crate) fn new(gpuspec: &GPUSpec) -> TextureBindery {
        TextureBindery {
            flat_to_binding: KeyedData::new(),
            position_to_flat: vec![None;gpuspec.max_textures() as usize],
            lru: VecDeque::new(),
            in_use: vec![],
            next_gl_index: 0
        }
    }

    fn get(&self, flat: &FlatId) -> anyhow::Result<Option<&Binding>> {
        Ok(self.flat_to_binding.get(flat).as_ref())
    }

    fn unbind(&mut self, flat: &FlatId) -> anyhow::Result<()> {
        if let Some(b) = self.flat_to_binding.remove(flat) {
            self.position_to_flat[b.position as usize] = None;
        }
        Ok(())
    }

    fn bind(&mut self, flat_id: &FlatId, index: usize) -> anyhow::Result<u32> {
        let gl_index = self.next_gl_index;
        let binding = Binding { position: index, gl_index };
        self.flat_to_binding.insert(flat_id,binding);
        self.next_gl_index += 1;
        self.position_to_flat[index as usize] = Some(flat_id.clone());
        Ok(gl_index)
    }

    fn set(&mut self, flat: &FlatId) -> anyhow::Result<Rebind> {
        if let Some(index) = self.lru.pop_front() {
            let mut old_texture = None;
            if let Some(old_id) = self.position_to_flat.get(index as usize).ok_or_else(|| err!("bad index"))?.clone() {
                self.unbind(&old_id)?;
                old_texture = Some(old_id);
            } else {
                self.in_use.push(index);
            }
            let new_index = self.bind(flat,index)?;
            Ok(Rebind {
                old_texture,
                new_texture: Some(flat.clone()),
                new_index
            })
        } else {
            bail!("too many textures bound");
        }
    }

    pub(crate) fn allocate(&mut self, flat: &FlatId) -> anyhow::Result<Rebind> {
        if let Some(b) = self.get(flat)? {
            return Ok(Rebind::cached(b.gl_index));
        }
        self.set(flat)
    }

    pub(crate) fn free(&mut self, flat: &FlatId) -> anyhow::Result<Rebind> {
        self.unbind(flat)?;
        Ok(Rebind::remove(flat.clone()))
    }

    pub(crate) fn clear(&mut self) {
        self.lru.extend(self.in_use.iter());
        self.in_use = vec![];
        self.next_gl_index = 0;
    }

    pub(crate) fn gl_index(&self, flat_id: &FlatId) -> anyhow::Result<u32> {
        Ok(self.flat_to_binding.get(flat_id).as_ref().ok_or_else(|| err!("no index assigned"))?.gl_index)
    }
}
