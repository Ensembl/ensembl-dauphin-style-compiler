use peregrine_draw::{ Message, PeregrineDom };

/* $ gets replaced by a random string each time this is run to avoid namespace collisions. */

const HTML : &str = r#"
    <div class="$-container">
        <div class="$-sticky"><canvas class="$-browser-canvas"></canvas></div>
        <div class="$-browser"></div>
    </div>
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

pub(crate) fn make_dom() -> Result<PeregrineDom,Message> {
    let window = web_sys::window().ok_or_else(|| Message::ConfusedWebBrowser(format!("cannot get window")))?;
    let document = window.document().ok_or_else(|| Message::ConfusedWebBrowser(format!("cannot get document")))?;
    let browser_el = document.get_element_by_id("browser").ok_or_else(|| Message::ConfusedWebBrowser(format!("cannot get canvas")))?;
    PeregrineDom::new(&browser_el,&HTML,&CSS)
}
