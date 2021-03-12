use wasm_bindgen::prelude::*;

mod integration {
    mod bell;
    pub(crate) mod pgcommander;
    pub(crate) mod pgdauphin;
    pub(crate) mod pgblackbox;
    pub(crate) mod pgchannel;
    pub(crate) mod pgconsole;
    pub(crate) mod pgintegration;
    mod stream;
}

mod run {
    pub(crate) mod draw;
    mod frame;
}

mod shape {
    pub(crate) mod core {
        pub(super) mod looper;
        pub(crate) mod glshape;
        pub(super) mod directcolourdraw;
        pub(super) mod geometrydata;
        pub(super) mod fixgeometry;
        pub(super) mod pagegeometry;
        pub(super) mod pingeometry;
        pub(super) mod spotcolourdraw;
        pub(super) mod tapegeometry;
        pub(crate) mod stage;
        pub(crate) mod text;
        pub(crate) mod texture;
        pub(crate) mod wigglegeometry;
    }

    pub(crate) mod layers {
        pub(crate) mod drawing;
        pub(crate) mod drawingzmenus;
        pub(super) mod consts;
        pub(crate) mod geometry;
        pub(crate) mod programstore;
        pub(crate) mod layer;
        pub(super) mod patina;
    }

    pub(crate) mod util {
        pub(super) mod iterators;
        pub(crate) mod glaxis;
        pub(super) mod quickvec;
        pub(crate) mod arrayutil;
    }
}

mod train {
    mod glcarriage;
    mod gltrain;
    mod gltrainset;

    pub(crate) use self::gltrainset::GlTrainSet;
}

mod util {
    pub(crate) mod ajax;
    pub(crate) mod error;
    pub(crate) mod safeelement;
}

mod webgl {
    pub(crate) mod canvas {
        pub(crate) mod bindery;
        pub(crate) mod drawingflats;
        pub(crate) mod flatplotallocator;
        pub(crate) mod flat;
        pub(crate) mod packer;
        pub(crate) mod flatstore;
        pub(crate) mod weave;
    }

    pub(crate) use canvas::weave::CanvasWeave;
    pub(crate) use canvas::flat::Flat;
    pub(crate) use canvas::flatstore::{ FlatId, FlatStore };
    pub(crate) use canvas::bindery::{ TextureBindery, TextureStore };
    pub(crate) use canvas::drawingflats::{ DrawingFlats, DrawingFlatsDrawable };
    pub(crate) use canvas::flatplotallocator::{ FlatPlotAllocator, FlatPlotRequestHandle };


    pub(super) mod gpuspec {
        pub(crate) mod glarity;
        pub(crate) mod gpuspec;
        pub(crate) mod precision;
        mod glsize;
    }

    pub(crate) use gpuspec::gpuspec::{ GPUSpec, Phase };
    pub(crate) use gpuspec::glarity::GLArity;
    pub(crate) use gpuspec::precision::Precision;

    mod program {
        pub(crate) mod compiler;
        pub(crate) mod texture;
        pub(crate) mod header;
        pub(crate) mod process;
        pub(crate) mod program;
        pub(crate) mod source;
        pub(crate) mod uniform;
        pub(crate) mod attribute;
        pub(crate) mod varying;
        pub(crate) mod statement;
        pub(crate) mod session;
    }

    mod stanza {
        pub(crate) mod array;
        pub(crate) mod builder;
        pub(crate) mod elements;
        pub(crate) mod stanza;
    }

    pub(crate) use stanza::elements::ProcessStanzaElements;
    pub(crate) use stanza::array::ProcessStanzaArray;
    pub(crate) use stanza::builder::{ ProcessStanzaBuilder, ProcessStanzaAddable };
    pub(crate) use stanza::stanza::ProcessStanza;
    pub(crate) use program::program::Program;
    pub(crate) use program::process::{ ProtoProcess, Process };
    pub(crate) use program::compiler::WebGlCompiler;
    pub(crate) use program::header::Header;
    pub(crate) use program::uniform::{ Uniform, UniformHandle };
    pub(crate) use program::attribute::{ Attribute, AttribHandle };
    pub(crate) use program::varying::Varying;
    pub(crate) use program::session::DrawingSession;
    pub(crate) use program::source::{ SourceInstrs };
    pub(crate) use program::statement::Statement;

    pub(crate) mod global;
    mod util;
}

use anyhow::{ self };
use commander::{ cdr_timer };
use crate::run::draw::{ PeregrineDraw, PeregrineDrawApi };
#[cfg(blackbox)]
use crate::integration::pgblackbox::{ pgblackbox_setup };
use crate::util::error::{ js_throw };
use peregrine_data::{ 
    StickId, Channel, ChannelLocation, Commander, Track
};
use peregrine_data::{ PeregrineConfig };
pub use url::Url;
use crate::integration::pgconsole::{ PgConsoleWeb };
use crate::util::error::{ js_option };

#[cfg(blackbox)]
use blackbox::{ blackbox_enable, blackbox_log };

async fn test(mut draw_api: PeregrineDraw) -> anyhow::Result<()> {
    draw_api.bootstrap(Channel::new(&ChannelLocation::HttpChannel(Url::parse("http://localhost:3333/api/data")?)))?;
    draw_api.add_track(Track::new("gene-pc-fwd"));
    //
    draw_api.set_stick(&StickId::new("homo_sapiens_GCA_000001405_27:1"));
    let mut pos = 2500000.;
    let mut scale = 20.;
    for _ in 0..20 {
        pos += 500000.;
        scale *= 0.1;
        draw_api.set_x(pos);
        draw_api.set_bp_per_screen(scale);
        cdr_timer(1000.).await;
    }
    Ok(())
}

fn test_fn() -> anyhow::Result<()> {
    let console = PgConsoleWeb::new(30,30.);
    let mut config = PeregrineConfig::new();
    config.set_f64("animate.fade.slow",500.);
    config.set_f64("animate.fade.fast",100.);
    let window = js_option(web_sys::window(),"cannot get window")?;
    let document = js_option(window.document(),"cannot get document")?;
    let canvas = js_option(document.get_element_by_id("trainset"),"canvas gone AWOL")?;
    let pg_web = js_throw(PeregrineDraw::new(config,console,canvas));
    let commander = pg_web.commander();
    commander.add_task("test",100,None,None,Box::pin(test(pg_web)));
    Ok(())
}

// Called when the wasm module is instantiated
#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    js_throw(test_fn());
    Ok(())
}

#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}