use peregrine_data::{Assets, BackendNamespace, Asset };
use peregrine_toolkit::error::Error;
use peregrine_toolkit::lock;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::Closure;
use web_sys::HtmlImageElement;
use crate::webgl::canvas::htmlcanvas::canvasinuse::CanvasAndContext;
use crate::webgl::canvas::composition::canvasitem::{CanvasItemSize, CanvasItem};
use crate::webgl::global::WebGlGlobal;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::mem;
use std::sync::{Arc, Mutex};

use super::imagecache::ImageCache;

// TODO padding measurements!

const PAD : u32 = 4;

fn pad(x: (u32,u32)) -> (u32,u32) {
    (x.0+PAD,x.1+PAD)
}

pub(crate) struct Bitmap {
    name: String,
    width: u32,
    height: u32,
    scale: u32,
    el: Arc<HtmlImageElement>,
    onload: Arc<Mutex<Option<Vec<Box<dyn FnOnce(&HtmlImageElement) + 'static>>>>>
}

impl Bitmap {
    fn new(assets: &Assets, image_cache: &ImageCache, channel: &BackendNamespace, name: &str) -> Result<Bitmap,Error> {
        let asset = assets.get(Some(channel),name).ok_or_else(|| Error::fatal(&format!("missing asset: {}",name)))?;
        let image = if let Some(cached) = image_cache.get(name) {
            cached
        } else {
            let fresh = Self::fresh(&asset,channel,name)?;
            image_cache.set(name,fresh.clone());
            fresh
        };
        Ok(Bitmap {
            name: name.to_string(),
            width: asset.metadata_u32("width").ok_or_else(|| Error::fatal(&format!("missing width: {}",name)))?,
            height: asset.metadata_u32("height").ok_or_else(|| Error::fatal(&format!("missing height: {}",name)))?,
            scale: asset.metadata_u32("scale").unwrap_or(100),
            el: Arc::new(image),
            onload: Arc::new(Mutex::new(None))
        })
    }

    fn fresh(asset: &Asset, _channel: &BackendNamespace, name: &str) -> Result<HtmlImageElement,Error> {
        let bytes = asset.bytes().ok_or_else(|| Error::fatal(&format!("missing asset: {}",name)))?.data_as_bytes().map_err(|_| Error::fatal("expected bytes"))?.clone();
        let ascii_data = base64::encode(&*bytes);
        let image = HtmlImageElement::new().map_err(|e| Error::fatal(&format!("creating image element: {:?}",e)))?;
        let queue : Arc<Mutex<Option<Vec<Box<dyn FnOnce(&HtmlImageElement) + 'static>>>>> = Arc::new(Mutex::new(Some(vec![])));
        let queue2 = queue.clone();
        let el = image.clone();
        let closure = Closure::once_into_js(move || {
            for cb in mem::replace(&mut *lock!(queue2),None).unwrap_or(vec![]) {
                cb(&el);
            }
        });
        image.set_onload(Some(&closure.as_ref().unchecked_ref()));
        image.set_src(&format!("data:image/png;base64,{0}",ascii_data));
        Ok(image)
    }

    pub(crate) fn onload<F>(&self, cb: F) where F: FnOnce(&HtmlImageElement) + 'static {
        let mut queue = lock!(self.onload);
        if let Some(queue) = &mut *queue {
            queue.push(Box::new(cb));
        } else {
            drop(queue);
            cb(&self.el);
        }
    }
}

fn dpr_round(size: u32, dpr: f32, scale: u32) -> u32 {
    ( ((size*100/scale) as f32) * dpr ).round() as u32
}

impl CanvasItem for Bitmap {
    fn calc_size(&self, gl: &mut WebGlGlobal) -> Result<CanvasItemSize,Error> {
        let dpr = gl.device_pixel_ratio();
        let size = (dpr_round(self.width,dpr,self.scale),dpr_round(self.height,dpr,self.scale));
        Ok(CanvasItemSize::new(size,(PAD,PAD)))
    }

    fn compute_hash(&self) -> Option<u64> {
        let mut hasher = DefaultHasher::new();
        self.name.hash(&mut hasher);
        Some(hasher.finish())
    }

    fn draw_on_bitmap(&self, canvas: &mut CanvasAndContext, origin: (u32,u32), size: (u32,u32)) -> Result<(),Error> {
        canvas.draw_png(pad(origin),size,self)?;
        Ok(())
    }
}

pub struct DrawingBitmap {
    image_cache: ImageCache,
    assets: Assets
}

impl DrawingBitmap {
    pub fn new(assets: &Assets, image_cache: &ImageCache) -> DrawingBitmap {
        DrawingBitmap {
            image_cache: image_cache.clone(),
            assets: assets.clone()
        }
    }

    pub(crate) fn make(&mut self, channel: &BackendNamespace, asset: &str) -> Result<Bitmap,Error> {
        Ok(Bitmap::new(&self.assets,&self.image_cache,channel,asset)?)
    }
}
