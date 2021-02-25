use crate::util::keyed::KeyedData;
use peregrine_core::{ Pen, DirectColour };
use crate::keyed_handle;
use super::store::{ CanvasStore };
use super::weave::{ CanvasWeave, CanvasRequestId };
use std::collections::HashMap;
use super::allocator::DrawingCanvasesAllocator;
use super::weave::DrawingCanvasesBuilder;

// TODO padding measurements!

fn draw_one(store: &CanvasStore, builder: &DrawingCanvasesBuilder, text: &str, request_id: &CanvasRequestId, colour: &DirectColour) -> anyhow::Result<()> {
    let origin = builder.origin(request_id);
    let size = builder.size(request_id);
    let canvas_id = builder.canvas(request_id);
    let canvas = store.get_main_canvas(&canvas_id)?;
    canvas.text(text,origin,size,colour)?;
    Ok(())
}

keyed_handle!(TextHandle);

struct Text {
    pen: Pen,
    text: String,
    size: Option<(u32,u32)>,
    colour: DirectColour,
    text_request: Option<CanvasRequestId>,
    mask_request: Option<CanvasRequestId>
}

impl Text {
    fn new(pen: &Pen, text: &str, colour: &DirectColour) -> Text {
        Text { pen: pen.clone(), text: text.to_string(), size: None, text_request: None, mask_request: None, colour: colour.clone() }
    }

    fn calc_size(&mut self, canvas_store: &mut CanvasStore) -> anyhow::Result<()> {
        let canvas = canvas_store.get_scratch_context(&CanvasWeave::Crisp,(16,16))?;
        canvas.set_font(&self.pen)?;
        self.size = Some(canvas.measure(&self.text)?);
        Ok(())
    }

    fn populate_allocator(&mut self, allocator: &mut DrawingCanvasesAllocator) -> anyhow::Result<()> {
        self.text_request = Some(allocator.allocate_area(&CanvasWeave::Crisp,self.size.unwrap())?);
        self.mask_request = Some(allocator.allocate_area(&CanvasWeave::Crisp,self.size.unwrap())?);
        Ok(())
    }

    fn build(&mut self, store: &CanvasStore, builder: &DrawingCanvasesBuilder) -> anyhow::Result<()> {
        draw_one(store,builder,&self.text,&self.text_request.clone().unwrap(),&self.colour.clone())?;
        draw_one(store,builder,&self.text,self.mask_request.as_ref().unwrap(),&self.colour)?;
        Ok(())
    }
}

pub struct DrawingText {
    texts: KeyedData<TextHandle,Text>
}

impl DrawingText {
    pub fn new() -> DrawingText {
        DrawingText {
            texts: KeyedData::new()
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
        for text in self.texts.values_mut() {
            text.populate_allocator(allocator)?;
        }
        Ok(())
    }

    pub fn build(&mut self, store: &CanvasStore, builder: &DrawingCanvasesBuilder) -> anyhow::Result<()> {
        for text in self.texts.values_mut() {
            text.build(store,builder)?;
        }
        Ok(())
    }
}
