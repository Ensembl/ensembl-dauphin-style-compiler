use anyhow::{ anyhow as err };
use crate::util::keyed::KeyedData;
use peregrine_core::{ Pen, DirectColour };
use crate::keyed_handle;
use super::flat::CanvasElement;
use super::store::{ CanvasStore, CanvasElementId };
use super::weave::{ CanvasWeave, CanvasRequestId, CanvasTextureAreas };
use std::collections::HashMap;
use super::allocator::DrawingCanvasesAllocator;
use super::weave::DrawingCanvasesBuilder;

// TODO padding measurements!

keyed_handle!(TextHandle);

struct Text {
    pen: Pen,
    text: String,
    text_origin: Option<(u32,u32)>,
    mask_origin: Option<(u32,u32)>,
    size: Option<(u32,u32)>,
    colour: DirectColour
}

impl Text {
    fn new(pen: &Pen, text: &str, colour: &DirectColour) -> Text {
        Text { pen: pen.clone(), text: text.to_string(), size: None, colour: colour.clone(), text_origin: None, mask_origin: None }
    }

    fn calc_size(&mut self, canvas_store: &mut CanvasStore) -> anyhow::Result<()> {
        let canvas = canvas_store.get_scratch_context(&CanvasWeave::Crisp,(16,16))?;
        canvas.set_font(&self.pen)?;
        self.size = Some(canvas.measure(&self.text)?);
        Ok(())
    }

    fn build(&mut self, canvas: &CanvasElement, text_origin: (u32,u32), mask_origin: (u32,u32)) -> anyhow::Result<()> {
        let size = self.size.unwrap();
        self.text_origin = Some(text_origin);
        self.mask_origin = Some(mask_origin);
        canvas.text(&self.text,text_origin,size,&self.colour)?;
        canvas.text(&self.text,mask_origin,size,&DirectColour(0,0,0))?;
        Ok(())
    }

    pub fn get_texture_areas(&self) -> anyhow::Result<CanvasTextureAreas> {
        Ok(CanvasTextureAreas {
            texture_origin: self.text_origin.as_ref().cloned().ok_or_else(|| err!("no origin"))?,
            mask_origin: self.mask_origin.as_ref().cloned().ok_or_else(|| err!("no origin"))?,
            size: self.size.as_ref().cloned().ok_or_else(|| err!("no size"))?
        })
    }
}

pub struct DrawingText {
    texts: KeyedData<TextHandle,Text>,
    request: Option<CanvasRequestId>
}

impl DrawingText {
    pub fn new() -> DrawingText {
        DrawingText {
            texts: KeyedData::new(),
            request: None
        }
    }

    pub fn add_text(&mut self, pen: &Pen, text: &str, colour: &DirectColour) -> TextHandle {
        self.texts.add(Text::new(pen,text,colour))
    }

    fn calc_sizes(&mut self, canvas_store: &mut CanvasStore) -> anyhow::Result<()> {
        /* All this to minimise font changes (which are slow) */
        let mut texts_by_pen = HashMap::new();
        for text in self.texts.values_mut() {
            texts_by_pen.entry(text.pen.clone()).or_insert_with(|| vec![]).push(text);
        }
        for (_,texts) in &mut texts_by_pen {
            for text in texts.iter_mut() {
                text.calc_size(canvas_store)?;
            }
        }
        Ok(())
    }

    pub(crate) fn populate_allocator(&mut self, canvas_store: &mut CanvasStore, allocator: &mut DrawingCanvasesAllocator) -> anyhow::Result<()> {
        self.calc_sizes(canvas_store)?;
        let mut sizes = vec![];
        for text in self.texts.values_mut() {
            let size = text.size.as_mut().unwrap().clone();
            /* mask and text */
            sizes.push(size);
            sizes.push(size);
        }
        self.request = Some(allocator.allocate_areas(&CanvasWeave::Crisp,&sizes));
        Ok(())
    }

    pub fn build(&mut self, store: &CanvasStore, builder: &DrawingCanvasesBuilder) -> anyhow::Result<()> {
        let mut origins = builder.origins(self.request.as_ref().unwrap());
        let mut origins_iter = origins.drain(..);
        let canvas_id = builder.canvas(self.request.as_ref().unwrap());
        let canvas = store.get_main_canvas(&canvas_id)?;
        for text in self.texts.values_mut() {
            let mask_origin = origins_iter.next().unwrap();
            let text_origin = origins_iter.next().unwrap();
            text.build(canvas,text_origin,mask_origin)?;
        }
        Ok(())
    }

    pub fn canvas_id(&self, builder: &DrawingCanvasesBuilder) -> anyhow::Result<CanvasElementId> {
        let request = self.request.as_ref().cloned().ok_or_else(|| err!("no such id"))?;
        Ok(builder.canvas(&request))
    }

    pub fn get_texture_areas(&self, handle: &TextHandle) -> anyhow::Result<CanvasTextureAreas> {
        self.texts.get(handle).get_texture_areas()
    }
}
