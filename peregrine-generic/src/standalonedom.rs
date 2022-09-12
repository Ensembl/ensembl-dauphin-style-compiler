use peregrine_draw::{ Message, PeregrineDom };
use web_sys::Element;

/* $ gets replaced by a random string each time this is run to avoid namespace collisions. */

const HTML2 : &str = r#"
    <canvas class="$-browser-canvas"></canvas>
"#;

pub(crate) fn make_dom_from_element(browser_el: &Element) -> Result<PeregrineDom,Message> {
    PeregrineDom::new(&browser_el,&HTML2)
}
