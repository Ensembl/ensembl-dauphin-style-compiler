use peregrine_draw::{ Message, PeregrineDom };
use web_sys::Element;

/* $ gets replaced by a random string each time this is run to avoid namespace collisions. */

const HTML : &str = r#"
    <div class="$-container">
        <div class="$-sticky"><canvas class="$-browser-canvas"></canvas></div>
        <div class="$-browser"></div>
    </div>
"#;

const HTML2 : &str = r#"
    <canvas class="$-browser-canvas"></canvas></div>
"#;

const CSS : &str = r#"
    .$-container {
        height: 100%;
        overflow: auto;
    }

    .$-browser {
        height: 1234px;
    }

    .$-sticky {
        position: sticky;
        top: 0;
        overflow: hidden;
        height: 100%;
    }
"#;

pub(crate) fn make_dom_from_element(browser_el: &Element) -> Result<PeregrineDom,Message> {
    PeregrineDom::new(&browser_el,&HTML2,&CSS)
}
