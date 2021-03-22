mod integration {
    mod bell;
    pub(crate) mod pgcommander;
    pub(crate) mod pgdauphin;
    pub(crate) mod pgchannel;
    pub(crate) mod pgintegration;
    mod stream;

    pub use self::pgcommander::PgCommanderWeb;
}

mod run {
    pub mod draw;
    mod frame;

    pub use self::draw::{ PeregrineDraw, PeregrineDrawApi };
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
        pub(crate) mod redrawneeded;
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
    pub(crate) mod message;
    pub(crate) mod safeelement;

    pub use self::ajax::PgAjax;
    pub use self::error::{ js_throw, js_option };
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

pub use crate::run::{ PeregrineDraw, PeregrineDrawApi };
pub use self::util::{ js_throw, js_option, PgAjax };
pub use crate::integration::PgCommanderWeb;
