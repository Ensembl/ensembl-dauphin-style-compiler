use commander::cdr_timer;
use peregrine_toolkit::{plumbing::oneshot::OneShot, log_extra, js::dommanip::{to_html, create_element, set_css, html_document, html_body, to_canvas, unique_element}};
use web_sys::{Document, Element, HtmlCanvasElement, HtmlElement };
use crate::{util::message::Message, PgCommanderWeb};

include!(concat!(env!("OUT_DIR"), "/env.rs"));

#[derive(Clone)]
pub struct PeregrineDom {
    canvas: HtmlCanvasElement,
    canvas_container: HtmlElement,
    viewport_element: HtmlElement,
    content_element: HtmlElement,
    document: Document,
    body: HtmlElement,
    device_pixel_ratio: f32,
    shutdown: OneShot
}

async fn check_for_shutdown(oneshot: &OneShot, element: &HtmlElement) -> Result<bool,Message> {
    if !element.is_connected() {
        log_extra!("shutting down");
        oneshot.run();
        Ok(true)
    } else {
        Ok(false)
    }
}

fn run_shutdown_detector(commander: &PgCommanderWeb, oneshot: &OneShot, element: &HtmlElement) ->Result<(),Message> {
    let oneshot = oneshot.clone();
    let element = element.clone();
    commander.add::<Message>("shutdown detector",20,None,None,Box::pin(async move {
        while !check_for_shutdown(&oneshot,&element).await? {
            cdr_timer(5000.).await;
        }
        Ok(())
    }));
    Ok(())
}

fn effective_dpr() -> f32 {
    let real_dpr = web_sys::window().unwrap().device_pixel_ratio() as f32;
    FORCE_DPR.unwrap_or(real_dpr)
}

fn confused<F,T>(cb: F) -> Result<T,Message> where F: FnOnce() -> Result<T,String> {
    cb().map_err(|x| Message::ConfusedWebBrowser(x))
}

macro_rules! map {
    ($($key:expr => $value:expr),*) => {
        {
            use std::collections::HashMap;

            let mut out = HashMap::new();
            $(
                out.insert($key,$value);
            )*
            out
        }
    }
}

impl PeregrineDom {
    pub fn new(el: &Element) -> Result<PeregrineDom,Message> {
        Self::new_inner(el).map_err(|e| Message::ConfusedWebBrowser(e))
    }

    fn new_inner(el: &Element) -> Result<PeregrineDom,String> {
        let device_pixel_ratio = effective_dpr();
        let shutdown = OneShot::new();
        /* For now, assume the nested document element mode */
        let canvas_container = to_html(el.clone())?;
        let content_element = to_html(unique_element(canvas_container.children())?
            .ok_or_else(|| format!("No inner element"))?)?;
        let canvas = to_html(create_element("canvas")?)?;
        /* container element must be position: relative so that canvas can be made absolute
         * so that the tops can align and the canvas be on screen. Slightly ugly in API-isolation
         * terms!
         */
        set_css(&canvas,  &map!(
            "position" => "sticky",
            "display" => "block",
            "top" => "0",
            "overflow" => "hidden"
            
        ))?;
        set_css(&content_element,&map!(
            "margin-top" => "0px"
        ))?;
        canvas_container.prepend_with_node_1(&canvas);
        let viewport_element = canvas_container.clone();
        /* */
        Ok(PeregrineDom {
            document: html_document()?,
            body: html_body()?,
            canvas: to_canvas(canvas)?,
            canvas_container,
            viewport_element,
            content_element,
            device_pixel_ratio,
            shutdown
        })
    }

    pub(crate) fn run_shutdown_detector(&self, commander:& PgCommanderWeb) {
        run_shutdown_detector(commander, &self.shutdown,&self.canvas_container);
    }

    pub(crate) fn shutdown(&self) -> &OneShot { &self.shutdown }

    pub(crate) fn canvas(&self) -> &HtmlCanvasElement { &self.canvas }
    pub(crate) fn viewport_element(&self) -> &HtmlElement { &self.viewport_element }
    pub(crate) fn document(&self) -> &Document { &self.document }
    pub(crate) fn body(&self) -> &HtmlElement { &self.body }
    pub(crate) fn device_pixel_ratio(&self) -> f32 { self.device_pixel_ratio }

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
