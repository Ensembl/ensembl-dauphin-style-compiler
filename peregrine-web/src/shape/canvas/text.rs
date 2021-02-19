use crate::util::keyed::KeyedData;
use peregrine_core::Pen;
use crate::keyed_handle;
use super::store::CanvasStore;
use super::weave::{ CanvasWeave, CanvasRequestId };
use std::collections::HashMap;
use super::allocator::DrawingCanvasesAllocator;
use super::weave::DrawingCanvasesBuilder;

keyed_handle!(TextHandle);

struct Text {
    pen: Pen,
    text: String,
    size: Option<(u32,u32)>,
    request: Option<CanvasRequestId>
}

impl Text {
    fn new(pen: &Pen, text: &str) -> Text {
        Text { pen: pen.clone(), text: text.to_string(), size: None, request: None }
    }

    fn calc_size(&mut self, canvas_store: &mut CanvasStore) -> anyhow::Result<()> {
        let canvas = canvas_store.get_scratch_context(&CanvasWeave::Crisp,(16,16))?;
        canvas.set_font(&self.pen);
        self.size = Some(canvas.measure(&self.text)?);
        Ok(())
    }

    fn populate_allocator(&mut self, allocator: &mut DrawingCanvasesAllocator) -> anyhow::Result<()> {
        self.request = Some(allocator.allocate_area(&CanvasWeave::Crisp,self.size.unwrap())?);
        Ok(())
    }

    fn draw(&mut self, canvas_store: &mut CanvasStore, builder: &DrawingCanvasesBuilder) -> anyhow::Result<()> {
        let origin = builder.origin(self.request.as_ref().unwrap());
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

    pub fn add_text(&mut self, pen: &Pen, text: &str) -> TextHandle {
        self.texts.add(Text::new(pen,text))
    }

    fn calc_sizes(&mut self, canvas_store: &mut CanvasStore) -> anyhow::Result<()> {
        /* All this to minimise font changes (slow) */
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

    pub(crate) fn populate_allocator(&mut self, allocator: &mut DrawingCanvasesAllocator) -> anyhow::Result<()> {
        for text in self.texts.values_mut() {
            text.populate_allocator(allocator)?;
        }
        Ok(())
    }

    pub fn draw(&mut self, canvas_store: &mut CanvasStore, builder: &DrawingCanvasesBuilder) -> anyhow::Result<()> {
        for text in self.texts.values_mut() {
            text.draw(canvas_store,builder)?;
        }
        Ok(())
    }
}
