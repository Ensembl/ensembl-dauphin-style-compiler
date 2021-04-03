use std::collections::{ HashSet };
use crate::webgl::{ FlatId, FlatStore };
use keyed::KeyedData;
use crate::webgl::GPUSpec;
use super::weave::CanvasWeave;
use web_sys::WebGlRenderingContext;
use web_sys::WebGlTexture;
use crate::webgl::util::handle_context_errors;
use crate::webgl::global::WebGlGlobal;
use crate::util::message::Message;
use crate::util::evictlist::EvictList;

fn apply_weave(context: &WebGlRenderingContext,weave: &CanvasWeave) -> Result<(),Message> {
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

fn create_texture(context: &WebGlRenderingContext,canvas_store: &FlatStore, our_data: &FlatId) -> Result<WebGlTexture,Message> {
    let canvas = canvas_store.get(our_data)?;
    let texture = context.create_texture().ok_or_else(|| Message::WebGLFailure("cannot create texture".to_string()))?;
    handle_context_errors(context)?;
    context.bind_texture(WebGlRenderingContext::TEXTURE_2D,Some(&texture));
    handle_context_errors(context)?;
    context.tex_image_2d_with_u32_and_u32_and_canvas( // wow
        WebGlRenderingContext::TEXTURE_2D,0,WebGlRenderingContext::RGBA as i32,WebGlRenderingContext::RGBA,
        WebGlRenderingContext::UNSIGNED_BYTE,canvas.element()?
    ).map_err(|e| Message::WebGLFailure(format!("cannot bind texture: {:?}",&e.as_string())))?;
    handle_context_errors(context)?;
    apply_weave(context,canvas.weave())?;
    Ok(texture)
}

pub struct TextureStore(KeyedData<FlatId,Option<WebGlTexture>>);

impl TextureStore {
    pub fn new() -> TextureStore {
        TextureStore(KeyedData::new())
    }

    fn add(&mut self, id: &FlatId, texture: WebGlTexture) {
        self.0.insert(id,texture);
    }
    
    fn get(&mut self, id: &FlatId) -> Result<Option<&WebGlTexture>,Message> {
        Ok(self.0.try_get(id).as_ref())
    }

    fn remove(&mut self, id: &FlatId) -> Result<WebGlTexture,Message> {
        self.0.remove(id).ok_or_else(|| Message::CodeInvariantFailed(format!("missing key C")))
    }
}

pub struct Rebind {
    old_texture: Option<FlatId>,
    new_texture: Option<FlatId>,
    new_index: u32
}

impl Rebind {
    fn new(old_texture: Option<FlatId>, new_texture: FlatId, new_index: u32) -> Rebind {
        Rebind { old_texture, new_texture: Some(new_texture), new_index }
    }

    fn cached(flat_id: &FlatId, index: u32) -> Rebind {
        Rebind { old_texture: None, new_texture: Some(flat_id.clone()), new_index: index }
    }

    fn remove(flat_id: FlatId) -> Rebind {
        Rebind { old_texture: Some(flat_id), new_texture: None, new_index: 0}
    }

    fn noop() ->Rebind {
        Rebind {old_texture: None, new_texture: None, new_index: 0 }
    }

    pub(crate) fn apply(&self, gl: &mut WebGlGlobal) -> Result<u32,Message> {
        if let Some(old_id) = &self.old_texture {
            let old_flat = gl.texture_store().remove(old_id)?;
            gl.context().delete_texture(Some(&old_flat));
            gl.handle_context_errors()?;
        }
        if let Some(new_id) = &self.new_texture {
            let texture = gl.texture_store().get(new_id)?;
            if texture.is_none() {
                let texture = create_texture(gl.context(),gl.flat_store(),new_id)?;
                gl.texture_store().add(new_id,texture);
            }
            gl.context().active_texture(WebGlRenderingContext::TEXTURE0 + self.new_index);
            gl.handle_context_errors()?;
            let context = gl.context().clone();
            let texture = gl.texture_store().get(new_id)?.unwrap();
            context.bind_texture(WebGlRenderingContext::TEXTURE_2D,Some(&texture));
            gl.handle_context_errors()?;
        }
        Ok(self.new_index)
    }
}

pub struct TextureBindery {
    lru: EvictList,
    active: Vec<FlatId>,
    available: HashSet<FlatId>,
    max_textures: u32,
    current_textures: u32,
    current_epoch: i64,
    next_gl_index: u32
}

impl TextureBindery {
    pub(crate) fn new(gpuspec: &GPUSpec) -> TextureBindery {
        let max_textures = gpuspec.max_textures();
        TextureBindery {
            lru: EvictList::new(),
            active: vec![],
            available: HashSet::new(),
            current_textures: 0,
            max_textures,
            current_epoch: 0,
            next_gl_index: 0
        }
    }

    pub fn allocate(&mut self, flat: &FlatId) -> Result<Rebind,Message> {
        self.active.push(flat.clone());
        self.lru.remove_item(flat);
        let our_gl_index = self.next_gl_index;
        self.next_gl_index += 1;
        if self.available.contains(flat) {
            return Ok(Rebind::cached(flat,our_gl_index));
        }
        self.available.insert(flat.clone());
        self.current_textures += 1;
        let mut old = None;
        if self.current_textures > self.max_textures {
            if let Some((_,old_item)) = self.lru.remove_oldest() {
                self.current_textures -= 1;
                self.available.remove(&old_item);
                old = Some(old_item);
            } else {
                return Err(Message::CodeInvariantFailed("too many textures bound".to_string()));
            }
        }
        Ok(Rebind::new(old,flat.clone(),our_gl_index))
    }

    pub fn free(&mut self, flat: &FlatId) -> Result<Rebind,Message> {
        if self.lru.remove_item(flat) { 
            self.available.remove(flat);
            Ok(Rebind::remove(flat.clone()))
        } else {
            Ok(Rebind::noop())
        }
    }

    pub fn clear(&mut self) {
        for flat in self.active.drain(..) {
            self.lru.insert(&flat,self.current_epoch);
        }
        self.current_epoch += 1;
        self.next_gl_index = 0;
    }
}
