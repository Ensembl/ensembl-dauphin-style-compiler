use web_sys::HtmlCanvasElement;

#[derive(Clone,Copy,PartialEq,Eq,Hash,Debug)]
pub enum CanvasWeave {
    Pixelate,
    Blur
}

#[derive(Clone)]
pub struct Canvas {
    element: HtmlCanvasElement,
    weave: CanvasWeave
}

impl Canvas {
    pub fn element(&self) -> &HtmlCanvasElement { &self.element }
    pub fn weave(&self) -> &CanvasWeave { &self.weave }
}