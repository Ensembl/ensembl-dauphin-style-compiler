use peregrine_toolkit::{plumbing::oneshot::OneShot, js::dommanip::{to_html, create_element, set_css, html_document, html_body, to_canvas, prepend_element}, map};
use peregrine_toolkit_async::sync::needed::Needed;
use web_sys::{Document, Element, HtmlCanvasElement, HtmlElement };
use crate::{util::message::Message, PgCommanderWeb};
use super::{shutdown::detect_shutdown, yposdetector::YPosDetector};

include!(concat!(env!("OUT_DIR"), "/env.rs"));

#[derive(Clone)]
pub(crate) struct PeregrineDom {
    canvas: HtmlCanvasElement,
    viewport_element: HtmlElement,
    content_element: HtmlElement,
    document: Document,
    body: HtmlElement,
    device_pixel_ratio: f32,
    shutdown: OneShot,
    ypos_detector: YPosDetector
}

fn effective_dpr() -> f32 {
    let real_dpr = web_sys::window().unwrap().device_pixel_ratio() as f32;
    FORCE_DPR.unwrap_or(real_dpr)
}

fn confused<F,T>(cb: F) -> Result<T,Message> where F: FnOnce() -> Result<T,String> {
    cb().map_err(|x| Message::ConfusedWebBrowser(x))
}

impl PeregrineDom {
    pub fn new(commander: &PgCommanderWeb, el: &Element, needed: &Needed) -> Result<PeregrineDom,Message> {
        let out = confused(|| {
            let viewport_element = to_html(el.clone())?;
            let content_element = to_html(create_element("div")?)?;
            prepend_element(&viewport_element,&content_element)?;
            let canvas = to_html(create_element("canvas")?)?;
            set_css(&canvas, &map!(
                "position" => "sticky",
                "display" => "block",
                "top" => "0",
                "overflow" => "hidden"
                
            ))?;
            set_css(&content_element,&map!(
                "margin-top" => "0px"
            ))?;
            prepend_element(&viewport_element,&canvas)?;
            let ypos_detector = YPosDetector::new(&viewport_element,needed).ok().unwrap(); // XXX
            let out = PeregrineDom {
                document: html_document()?,
                body: html_body()?,
                canvas: to_canvas(canvas)?,
                viewport_element,
                content_element,
                device_pixel_ratio: effective_dpr(),
                shutdown: OneShot::new(),
                ypos_detector
            };
            Ok(out)
        })?;
        detect_shutdown(commander, &out.shutdown,&out.viewport_element)?;
        Ok(out)
    }

    pub(crate) fn shutdown(&self) -> &OneShot { &self.shutdown }

    pub(crate) fn canvas(&self) -> &HtmlCanvasElement { &self.canvas }
    pub(crate) fn viewport_element(&self) -> &HtmlElement { &self.viewport_element }
    pub(crate) fn document(&self) -> &Document { &self.document }
    pub(crate) fn body(&self) -> &HtmlElement { &self.body }
    pub(crate) fn device_pixel_ratio(&self) -> f32 { self.device_pixel_ratio }
    pub(crate) fn ypos_detector(&self) -> &YPosDetector { &self.ypos_detector }

    pub(crate) fn set_content_height(&self, height: u32) -> Result<(),Message> {
        confused(|| {
            set_css(&self.content_element,&map!(
                "height" => format!("{}px",height)
            ))?;
            Ok(())
        })
    }

    pub(crate) fn set_canvas_size(&self, width: u32, height: u32) -> Result<(),Message> {
        confused(|| {
            self.canvas.set_width((width as f32*self.device_pixel_ratio) as u32);
            self.canvas.set_height((height as f32*self.device_pixel_ratio) as u32);
            set_css(&self.canvas,&map!(
                "height" => format!("{}px",height),
                "width" => format!("{}px",width)
            ))?;
            set_css(&self.content_element,&map!(
                "margin-top" => format!("-{}px",height)
            ))?;
            Ok(())
        })
    }
}
