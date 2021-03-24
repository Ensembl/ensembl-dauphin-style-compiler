use peregrine_draw::{ Message, PeregrineDom };

const HTML : &str = r#"
    <div class="$-container">
        <div class="$-sticky"></div>
        <div class="$-browser"><canvas class="$-browser-canvas"></canvas></div>
    </div>
"#;

const CSS : &str = r#"
    .$-browser {
        height: 1234px;
    }

    .$-sticky {
        background: url(http://www.ensembl.info/wp-content/uploads/2018/02/weird_genomes_4tile2.png) repeat;
        position: sticky;
        top: 0;
        height: 100%;
    }
"#;

pub(crate) fn make_dom() -> Result<PeregrineDom,Message> {
    let window = web_sys::window().ok_or_else(|| Message::ConfusedWebBrowser(format!("cannot get window")))?;
    let document = window.document().ok_or_else(|| Message::ConfusedWebBrowser(format!("cannot get document")))?;
    let browser_el = document.get_element_by_id("browser").ok_or_else(|| Message::ConfusedWebBrowser(format!("cannot get canvas")))?;
    PeregrineDom::new(&browser_el,&HTML,&CSS)
}
