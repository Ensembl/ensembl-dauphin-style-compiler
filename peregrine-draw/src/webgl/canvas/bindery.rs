use crate::webgl::{ FlatId, FlatStore };
use crate::webgl::GPUSpec;
use super::weave::CanvasWeave;
use web_sys::WebGlRenderingContext;
use web_sys::WebGlTexture;
use crate::webgl::util::handle_context_errors;
use crate::util::message::Message;

/* We don't want to recreate textures if we don't have to. The main reason for this is repeated draws of multiple
 * programs (likely from adjacent carriages).
 *
 * Textures are in one of three states: ACTIVE, AVAILABLE, or SLEEPING. Textures which are ACTIVE or AVAILABLE have
 * been created in WebGL, SLEEPING have not. ACTIVE are needed for this render but AVAILABLE are not (yet!). Should
 * insufficent texture slots be available, we can first take an AVAILABLE texture and unbind it to free the space.
 * On the other hand, if we wish use an AVAILABLE texture, we can do so without rebinding.
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

pub(crate) struct TextureBindery {
    available_or_active: Vec<FlatId>,
    max_textures: usize,
    current_epoch: i64,
    next_gl_index: u32
}

impl TextureBindery {
    pub(crate) fn new(gpuspec: &GPUSpec) -> TextureBindery {
        let max_textures = gpuspec.max_textures() as usize;
        TextureBindery {
            available_or_active: vec![],
            max_textures,
            current_epoch: 0,
            next_gl_index: 0
        }
    }

    fn find_victim(&mut self, flat_store: &mut FlatStore) -> Result<FlatId,Message> {
        let flats = self.available_or_active.iter().cloned().collect::<Vec<_>>();
        for (i,flat_id) in flats.iter().enumerate() {
            if !*flat_store.get_mut(&flat_id)?.is_active() {
                self.available_or_active.swap_remove(i);
                return Ok(flat_id.clone());
            }
        }
        return Err(Message::CodeInvariantFailed("too many textures bound".to_string()));
    }

    fn make_one_unavailable(&mut self, flat_store: &mut FlatStore) -> Result<(),Message> {
        let old_item = self.find_victim(flat_store)?;
        flat_store.get_mut(&old_item)?.set_gl_texture(None);
        Ok(())
    }

    fn make_available(&mut self, flat: &FlatId, flat_store: &mut FlatStore, context: &WebGlRenderingContext) -> Result<(),Message> {
        if self.available_or_active.len() >= self.max_textures {
            self.make_one_unavailable(flat_store)?;
        }
        self.available_or_active.push(flat.clone());
        let texture = create_texture(context,flat_store,flat)?;
        flat_store.get_mut(flat)?.set_gl_texture(Some(texture));
        Ok(())
    }

    pub(crate) fn allocate(&mut self, flat_id: &FlatId, flat_store: &mut FlatStore, context: &WebGlRenderingContext) -> Result<u32,Message> {
        /* Promote to AVAILABLE if SLEEPING */
        if !self.available_or_active.contains(flat_id) {
            self.make_available(flat_id,flat_store,context)?;
        }
        /* Promote from AVAILABLE to ACTIVE */
        *flat_store.get_mut(flat_id)?.is_active() = true;
        /* Allocate a gl index for this program */
        let our_gl_index = self.next_gl_index;
        self.next_gl_index += 1;
        /* Actually bind it */
        context.active_texture(WebGlRenderingContext::TEXTURE0 + our_gl_index);
        handle_context_errors(context)?;
        let texture = flat_store.get(flat_id)?.get_gl_texture().unwrap();
        context.bind_texture(WebGlRenderingContext::TEXTURE_2D,Some(texture.texture()));
        handle_context_errors(context)?;
        Ok(our_gl_index)
    }

    pub(crate) fn free(&mut self, flat: &FlatId, flat_store: &mut FlatStore) -> Result<(),Message> {
        if let Some(pos) = self.available_or_active.iter().position(|id| id == flat) {
            self.available_or_active.swap_remove(pos);
        }
        flat_store.get_mut(flat)?.set_gl_texture(None);
        Ok(())
    }

    pub(crate) fn clear(&mut self, flat_store: &mut FlatStore) -> Result<(),Message> {
        for flat_id in &self.available_or_active {
            let is_active = flat_store.get_mut(&flat_id)?.is_active();
            if *is_active {
                *flat_store.get_mut(&flat_id)?.is_active() = false;
            }
        }
        self.current_epoch += 1;
        self.next_gl_index = 0;
        Ok(())
    }
}
