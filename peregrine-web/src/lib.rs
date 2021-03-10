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
    pub(crate) mod web;
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
use crate::run::web::PeregrineWeb;
#[cfg(blackbox)]
use crate::integration::pgblackbox::{ pgblackbox_setup };
use crate::util::error::{ js_throw };
use peregrine_core::{ 
    StickId, PeregrineCore, Channel, ChannelLocation, Commander, Track
};
pub use url::Url;

#[cfg(blackbox)]
use blackbox::{ blackbox_enable, blackbox_log };

async fn test(api: PeregrineCore) -> anyhow::Result<()> {
    api.add_track(Track::new("gene-pc-fwd"));
    api.set_stick(&StickId::new("homo_sapiens_GCA_000001405_27:1"));
    let mut pos = 2500000.;
    let mut scale = 20.;
    for _ in 0..20 {
        pos += 500000.;
        scale += 0.1;
        api.set_position(pos);
        api.set_scale(scale);
        cdr_timer(1000.).await;
    }
    Ok(())
}

fn test_fn() -> anyhow::Result<()> {
    let pg_web = js_throw(PeregrineWeb::new());
    pg_web.data_api.backend_bootstrap(Channel::new(&ChannelLocation::HttpChannel(Url::parse("http://localhost:3333/api/data")?)));
    pg_web.commander.add_task("test",100,None,None,Box::pin(test(pg_web.data_api.clone())));
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