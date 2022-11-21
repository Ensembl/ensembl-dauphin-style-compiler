use std::{hash::{Hash, Hasher}, sync::Mutex};
use peregrine_data::{PenGeometry, DirectColour, Background};
use peregrine_toolkit::{lock, error::Error};
use crate::{util::fonts::Fonts, webgl::CanvasAndContext};

/* \0cXXYYZZ -- override colour X,Y,Z (hex)
 * \0c-      -- reset colour
 */

// XXX dedup from flat: generally move all text stuff into here
fn pen_to_font(pen: &PenGeometry, bitmap_multiplier: f64) -> String {
    format!("{}px {}",(pen.size_in_webgl() * bitmap_multiplier).round(),pen.name())
}

const PAD : u32 = 4;

fn pad(x: (u32,u32)) -> (u32,u32) {
    (x.0+PAD,x.1+PAD)
}

#[derive(Hash)]
pub struct StructuredTextGroup<'a>(&'a PenGeometry);

struct StructuredTextPart {
    pen: Option<PenGeometry>,
    text: String,
    colour: Option<DirectColour>,
    measured: Mutex<Option<(u32,u32)>>
}

impl StructuredTextPart {
    fn measure(&self, canvas: &mut CanvasAndContext, parent: &StructuredText) -> Result<(u32, u32),Error> {
        if let Some(measured) = *lock!(self.measured) {
            return Ok(measured);
        }
        let pen = self.pen.as_ref().unwrap_or(&parent.pen);
        canvas.set_font(pen)?;
        let size = canvas.measure(&self.text)?;
        *lock!(self.measured) = Some(size);
        Ok(size)
    }

    fn draw(&self, canvas: &mut CanvasAndContext, text_origin: (u32,u32), parent: &StructuredText) -> Result<(),Error> {
        let pen = self.pen.as_ref().unwrap_or(&parent.pen);
        let colour = self.colour.as_ref().unwrap_or(&parent.colour);
        canvas.set_font(pen)?;
        canvas.text(&self.text,text_origin,colour)?;
        Ok(())
    }
}

impl Hash for StructuredTextPart {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.pen.hash(state);
        self.text.hash(state);
        self.colour.hash(state);
    }
}

impl Clone for StructuredTextPart {
    fn clone(&self) -> Self {
        Self {
            pen: self.pen.clone(), 
            text: self.text.clone(), 
            colour: self.colour.clone(),
            measured: Mutex::new(lock!(self.measured).clone())
        }
    }
}

#[derive(Hash)]
pub struct StructuredText {
    pen: PenGeometry,
    parts: Vec<StructuredTextPart>,
    colour: DirectColour,
    background: Option<Background>
}

fn u32_to_directcolour(input: u32) -> DirectColour {
    DirectColour(((input>>16)&0xFF) as u8,((input>>8)&0xFF) as u8,(input&0xFF) as u8,255)
}

fn parse_structured_text(text: &str) -> Vec<StructuredTextPart> {
    let mut out = vec![
        StructuredTextPart {
            pen: None,
            text: "".to_string(),
            colour: None,
            measured: Mutex::new(None)
        }
    ];
    let mut bs : Option<String> = None;
    for c in text.chars() {
        let mut cancel_bs = false;
        if let Some(bs) = &mut bs {
            bs.push(c);
            cancel_bs = true;
            if *bs == "c-" {
                /* cancel colour */
                let mut new = out.last().unwrap().clone();
                new.colour = None;
                new.text = "".to_string();
                out.push(new);
            } else if bs.starts_with("c") && bs.len() == 7 {
                /* set colour */
                if let Ok(colour_hex) = u32::from_str_radix(&bs[1..],16) {
                    let mut new = out.last().unwrap().clone();
                    new.colour = Some(u32_to_directcolour(colour_hex));
                    new.text = "".to_string();
                    out.push(new);
                }
            } else if bs == "" || (bs.starts_with("c") && bs.len()<7) {
                cancel_bs = false;
            }
        } else if c == '\0' {
            bs = Some("".to_string());
        } else {
            out.last_mut().unwrap().text.push(c);
        }
        if cancel_bs {
            bs = None;
        }
    }
    out.drain(..).filter(|x| x.text.len()>0).collect::<Vec<_>>()
}

impl StructuredText {
    pub(crate) fn new(pen: &PenGeometry, text: &str, colour: &DirectColour, background: &Option<Background>) -> StructuredText {
        let parts = parse_structured_text(text);
        StructuredText {
            pen: pen.clone(), parts, colour: colour.clone(), background: background.clone()
        }
    }

    pub(crate) async fn prepare(&self, fonts: &Fonts, bitmap_multiplier: f64) {
        let new_font = pen_to_font(&self.pen,bitmap_multiplier);
        fonts.load_font(&new_font).await;
    }

    pub(crate) fn measure(&self, canvas: &mut CanvasAndContext) -> Result<(u32, u32),Error> {
        let mut size = (0,0);
        for part in &self.parts {
            let part_size = part.measure(canvas,self)?;
            size.0 += part_size.0;
            size.1 = size.1.max(part_size.1);
        }
        Ok(size)
    }

    pub(crate) fn group<'a>(&'a self) -> StructuredTextGroup<'a> {
        StructuredTextGroup(&self.pen)
    }

    pub(crate) fn draw(&mut self, canvas: &mut CanvasAndContext, text_origin: (u32,u32), size: (u32,u32)) -> Result<(),Error> {
        canvas.set_font(&self.pen)?;
        let background = self.background.clone().unwrap_or_else(|| Background::none());
        canvas.background(text_origin,size,&background,false)?;
        let mut offset = pad(text_origin);
        for part in &self.parts {
            part.draw(canvas,offset,self)?;
            offset.0 += part.measure(canvas,self)?.0;
        }
        Ok(())
    }    
}
