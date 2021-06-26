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

/* We don't want to recreate textures if we don't have to. The main reason for this is repeated draws of multiple
 * programs (likely from adjacent carriages).
 *
 * Textures are in one of three states: ACTIVE, AVAILABLE, or SLEEPING. Textures which are ACTIVE or AVAILABLE have
 * been created in WebGL, SLEEPING have not. ACTIVE are needed for this render but AVAILABLE are not (yet!). Should
 * insufficent texture slots be available, we can first take an AVAILABLE texture and unbind it to free the space.
 * On the other hand, if we wish use an AVAILABLE texture, we can do so without rebinding.
 *
 *
 * There are three operations during draw.
 * allocate(): a texture needs to be bound, ie transferred from SLEEPING or AVAILABLE to ACTIVE.
 * free(): a flat is no longer required, it should be moved from AVAILABLE or ACTIVE into SLEEPING.
 * clear(): the drawing is done. All ACTIVE flats should be made AVAILABLE.
 *
 * There are four WebGL operations which can take place during an attempted rebinding:
 * noop: do nothing
 * activate: use this texture (SLEEPING,AVAILABLE->ACTIVE).
 * remove: during free(), webgl is told to forget about this texture. (AVAILABLE,ACTIVE->SLEEPING).
 * remove_activate: both of these ops on two textures (AVAILABLE,ACTIVE->SLEEPING; SLEEPING,AVAILABLE->ACTIVE).
 *
 * An LRU called .lru contains AVAILABLE textures.
 * A Vec called .active contains ACTIVE textures.
 * A HashSet called .available contains AVAILABLE and ACTIVE TEXTURES.
 *
 * To activate a flat, first we remove it from .lru and add it to the .active, marking it as ACTIVE.
 * If the flat is in .available then it is AVAILABLE. In this case it's removed and activate called.
 * 
 */

fn apply_weave(context: &WebGlRenderingContext,weave: &CanvasWeave) -> Result<(),Message> {
    let (minf,magf,wraps,wrapt) = match weave {
        CanvasWeave::Crisp =>
            (WebGlRenderingContext::LINEAR,WebGlRenderingContext::LINEAR,
                WebGlRenderingContext::REPEAT,WebGlRenderingContext::REPEAT),
        CanvasWeave::Fuzzy =>
            (WebGlRenderingContext::LINEAR,WebGlRenderingContext::LINEAR,
                WebGlRenderingContext::REPEAT,WebGlRenderingContext::REPEAT),
        CanvasWeave::Heraldry => 
            (WebGlRenderingContext::NEAREST,WebGlRenderingContext::NEAREST,
                WebGlRenderingContext::REPEAT,WebGlRenderingContext::REPEAT),
        CanvasWeave::HorizStack => 
            (WebGlRenderingContext::NEAREST,WebGlRenderingContext::NEAREST,
                WebGlRenderingContext::REPEAT,WebGlRenderingContext::REPEAT),
        CanvasWeave::VertStack => 
            (WebGlRenderingContext::NEAREST,WebGlRenderingContext::NEAREST,
                WebGlRenderingContext::REPEAT,WebGlRenderingContext::REPEAT),        
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

fn create_texture(context: &WebGlRenderingContext,canvas_store: &FlatStore, our_data: &FlatId) -> Result<SelfManagedWebGlTexture,Message> {
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
    Ok(SelfManagedWebGlTexture::new(texture,context))
}

pub struct SelfManagedWebGlTexture(WebGlTexture,WebGlRenderingContext);

impl SelfManagedWebGlTexture {
    pub fn new(texture: WebGlTexture, context: &WebGlRenderingContext) -> SelfManagedWebGlTexture {
        SelfManagedWebGlTexture(texture,context.clone())
    }

    pub fn texture(&self) -> &WebGlTexture { &self.0 }
}

impl Drop for SelfManagedWebGlTexture {
    fn drop(&mut self) {
        self.1.delete_texture(Some(&self.0));
        // XXX errors
    }
}

pub struct Rebind {
    old_texture: Option<FlatId>,
    new_texture: Option<FlatId>,
    new_index: u32
}

impl Rebind {
    fn remove_activate(old_texture: Option<FlatId>, new_texture: FlatId, new_index: u32) -> Rebind {
        Rebind { old_texture, new_texture: Some(new_texture), new_index }
    }

    fn activate(flat_id: &FlatId, index: u32) -> Rebind {
        Rebind { old_texture: None, new_texture: Some(flat_id.clone()), new_index: index }
    }

    fn remove(flat_id: FlatId) -> Rebind {
        Rebind { old_texture: Some(flat_id), new_texture: None, new_index: 0}
    }

    fn noop() -> Rebind {
        Rebind {old_texture: None, new_texture: None, new_index: 0 }
    }

    pub(crate) fn apply(&self, gl: &mut WebGlGlobal) -> Result<u32,Message> {
        if let Some(old_id) = &self.old_texture {
            gl.flat_store_mut().get_mut(old_id)?.set_gl_texture(None);
        }
        if let Some(new_id) = &self.new_texture {
            let texture = gl.flat_store_mut().get(new_id)?.get_gl_texture();
            if texture.is_none() {
                let texture = create_texture(gl.context(),gl.flat_store(),new_id)?;
                gl.flat_store_mut().get_mut(new_id)?.set_gl_texture(Some(texture));
            }
            gl.context().active_texture(WebGlRenderingContext::TEXTURE0 + self.new_index);
            gl.handle_context_errors()?;
            let context = gl.context().clone();
            let texture = gl.flat_store_mut().get(new_id)?.get_gl_texture().unwrap();
            context.bind_texture(WebGlRenderingContext::TEXTURE_2D,Some(texture.texture()));
            gl.handle_context_errors()?;
        }
        Ok(self.new_index)
    }
}

pub(crate) struct TextureBindery {
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

    pub(crate) fn allocate(&mut self, flat: &FlatId) -> Result<Rebind,Message> {
        self.active.push(flat.clone());
        self.lru.remove_item(flat);
        let our_gl_index = self.next_gl_index;
        self.next_gl_index += 1;
        if self.available.contains(flat) {
            return Ok(Rebind::activate(flat,our_gl_index));
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
        Ok(Rebind::remove_activate(old,flat.clone(),our_gl_index))
    }

    pub(crate) fn free(&mut self, flat: &FlatId) -> Result<Rebind,Message> {
        if self.lru.remove_item(flat) { 
            self.available.remove(flat);
            self.current_textures -= 1;
            Ok(Rebind::remove(flat.clone()))
        } else {
            Ok(Rebind::noop())
        }
    }

    pub(crate) fn clear(&mut self) {
        for flat in self.active.drain(..) {
            self.lru.insert(&flat,self.current_epoch);
        }
        self.current_epoch += 1;
        self.next_gl_index = 0;
    }
}
