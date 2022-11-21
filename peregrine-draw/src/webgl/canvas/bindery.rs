use crate::webgl::GPUSpec;
use super::canvasinuse::CanvasInUse;
use super::weave::CanvasWeave;
use peregrine_toolkit::error::Error;
use web_sys::WebGlRenderingContext;
use web_sys::WebGlTexture;
use crate::webgl::util::handle_context_errors2;

/* 2D drawing contexts, must be converted to WebGL "textures" "bound" to the GPU. This process
 * allocates a handle to the texture (a WebGlTexture). In addition, textures must be bound to
 * indexes for the GPU on each run. This isn't a complex operation, but we do it here.
 *
 * This has shown itself to be a slow operation. It is best if we keep textures bound even
 * between runs of a program. If we do this we will end up with the situation at the start of the
 * program run that some random selection of potentially usable canvases will have been made
 * textures.
 * 
 * This store has a Vec, available_or_active, which contains all current textures. Each is stored
 * wrapped in a dropper, which unallocates its texture when it is dropped.
 * 
 */

/* We don't want to recreate textures if we don't have to. The main reason for this is repeated
 * draws of multiple programs (likely from adjacent carriages).
 *
 * Textures are in one of three states: ACTIVE, AVAILABLE, or SLEEPING. Textures which are ACTIVE 
 * or AVAILABLE have been created in WebGL, SLEEPING have not. ACTIVE are needed for this render
 * but AVAILABLE are not (yet!). Should insufficent texture slots be available, we can first take 
 * an AVAILABLE texture and unbind it to free the space. On the other hand, if we wish use an
 * AVAILABLE texture, we can do so without rebinding.
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

fn apply_weave(context: &WebGlRenderingContext,weave: &CanvasWeave) -> Result<(),Error> {
    let (minf,magf,wraps,wrapt) = match weave {
        CanvasWeave::Crisp =>
            (WebGlRenderingContext::NEAREST,WebGlRenderingContext::NEAREST,
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
    handle_context_errors2(context)?;
    Ok(())
}

fn create_texture(context: &WebGlRenderingContext, our_data: &CanvasInUse) -> Result<SelfManagedWebGlTexture,Error> {
    let (element,weave) = our_data.retrieve(|flat| {
        (flat.element().cloned(),flat.weave().clone())
    });
    let texture = context.create_texture().ok_or_else(|| Error::fatal("cannot create texture"))?;
    handle_context_errors2(context)?;
    context.bind_texture(WebGlRenderingContext::TEXTURE_2D,Some(&texture));
    handle_context_errors2(context)?;
    context.tex_image_2d_with_u32_and_u32_and_canvas( // wow
        WebGlRenderingContext::TEXTURE_2D,0,WebGlRenderingContext::RGBA as i32,WebGlRenderingContext::RGBA,
        WebGlRenderingContext::UNSIGNED_BYTE,&element?
    ).map_err(|e| Error::fatal(&format!("cannot bind texture: {:?}",&e.as_string())))?;
    handle_context_errors2(context)?;
    apply_weave(context,&weave)?;
    Ok(SelfManagedWebGlTexture::new(texture,context))
}

pub struct SelfManagedWebGlTexture {
    texture: WebGlTexture,
    context: WebGlRenderingContext,
    gl_index: Option<u32>
}

impl SelfManagedWebGlTexture {
    pub fn new(texture: WebGlTexture, context: &WebGlRenderingContext) -> SelfManagedWebGlTexture {
        SelfManagedWebGlTexture {
            texture,
            context: context.clone(),
            gl_index: None
        }
    }

    pub fn texture(&self) -> &WebGlTexture { &self.texture }
    pub fn gl_index(&self) -> u32 { self.gl_index.unwrap() }
}

impl Drop for SelfManagedWebGlTexture {
    fn drop(&mut self) {
        self.context.delete_texture(Some(&self.texture));
        // XXX errors
    }
}

pub(crate) struct TextureBindery {
    available_or_active: Vec<CanvasInUse>,
    max_textures: usize,
    next_gl_index: u32
}

impl TextureBindery {
    pub(crate) fn new(gpuspec: &GPUSpec) -> TextureBindery {
        let max_textures = gpuspec.max_textures() as usize;
        TextureBindery {
            available_or_active: vec![],
            max_textures,
            next_gl_index: 0
        }
    }

    fn find_victim(&mut self) -> Result<CanvasInUse,Error> {
        let flats = self.available_or_active.iter().cloned().collect::<Vec<_>>();
        for (i,flat_id) in flats.iter().enumerate() {
            if !flat_id.modify(|flat| *flat.is_active()) {
                self.available_or_active.swap_remove(i);
                return Ok(flat_id.clone());
            }
        }
        return Err(Error::fatal("too many textures bound"));
    }

    fn make_one_unavailable(&mut self) -> Result<(),Error> {
        let old_item = self.find_victim()?;
        old_item.modify(|flat| { flat.set_gl_texture(None) });
        Ok(())
    }

    fn make_available(&mut self, flat: &CanvasInUse, context: &WebGlRenderingContext) -> Result<(),Error> {
        if self.available_or_active.len() >= self.max_textures {
            self.make_one_unavailable()?;
        }
        self.available_or_active.push(flat.clone());
        let texture = create_texture(context,flat)?;
        flat.modify(|flat| { flat.set_gl_texture(Some(texture)) });
        Ok(())
    }

    pub(crate) fn allocate(&mut self, flat_id: &CanvasInUse, context: &WebGlRenderingContext) -> Result<(),Error> {
        /* Promote to AVAILABLE if SLEEPING */
        if !self.available_or_active.contains(flat_id) {
            self.make_available(flat_id,context)?;
        }
        /* Promote from AVAILABLE to ACTIVE */
        flat_id.modify(|flat| { *flat.is_active() = true; });
        /* Allocate a gl index for this program */
        let our_gl_index = self.next_gl_index;
        self.next_gl_index += 1;
        /* Actually bind it */
        context.active_texture(WebGlRenderingContext::TEXTURE0 + our_gl_index);
        handle_context_errors2(context)?;
        let texture = flat_id.modify(|flat| { 
            let texture = flat.get_gl_texture_mut().unwrap();
            texture.gl_index = Some(our_gl_index);
            flat.get_gl_texture().unwrap().texture().clone()
        });
        context.bind_texture(WebGlRenderingContext::TEXTURE_2D,Some(&texture));
        handle_context_errors2(context)?;
        Ok(())
    }

    pub(crate) fn free(&mut self, flat: &CanvasInUse) -> Result<(),Error> {
        if let Some(pos) = self.available_or_active.iter().position(|id| id == flat) {
            self.available_or_active.swap_remove(pos);
        }
        flat.modify(|flat| { flat.set_gl_texture(None) });
        Ok(())
    }

    pub(crate) fn clear(&mut self) -> Result<(),Error> {
        for flat_id in &self.available_or_active {
            let is_active = flat_id.modify(|flat| *flat.is_active());
            if is_active {
                flat_id.modify(|flat| { *flat.is_active() = false; });
            }
        }
        self.next_gl_index = 0;
        Ok(())
    }
}
