mod input {
    mod core {
        pub mod input;
    }

    mod low {
        mod pointer {
            pub(crate) mod cursor;
            mod drag;
            pub(super) mod pinch;
            pub(super) mod pointer;   
        }

        mod event;
        pub(crate) mod keyboardinput;
        pub(crate) mod mouseinput;
        pub(crate) mod lowlevel; 
        pub(crate) mod keyspec;
        pub(crate) mod modifiers;
        pub mod mapping;
    }

    mod translate {
        pub(crate) mod physics;
        pub(crate) mod debug;

        pub use self::physics::Physics;
    }

    pub use self::core::input::{ Input, InputEvent, InputEventKind, Distributor };
}

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
    pub mod api;
    mod config;
    mod dom;
    mod globalconfig;
    pub mod inner;
    mod frame;
    pub(crate) mod progress;
    mod size;

    pub use self::config::{ PgPeregrineConfig, PgConfigKey, CursorCircumstance };
    pub use self::globalconfig::PeregrineConfig;
    pub use self::dom::PeregrineDom;
    pub use self::api::{ PeregrineAPI };
    pub use self::inner::{ PeregrineInnerAPI };
}

mod shape {
    pub(crate) mod core {
        pub(crate) mod prepareshape;
        pub(crate) mod drawshape;
        pub(super) mod directcolourdraw;
        pub(super) mod geometrydata;
        pub(super) mod spotcolourdraw;
        pub(super) mod tracktriangles;
        pub(crate) mod flatdrawing;
        pub(crate) mod spectre;
        pub(crate) mod spectraldrawing;
        pub(crate) mod spectremanager;
        pub(crate) mod text;
        pub(crate) mod texture;
        pub(crate) mod wigglegeometry;
    }

    pub(crate) mod heraldry {
        pub(super) mod bardots;
        pub(crate) mod heraldry;
    }

    pub(crate) mod layers {
        pub(crate) mod drawing;
        pub(crate) mod drawingzmenus;
        pub(crate) mod consts;
        pub(crate) mod geometry;
        pub(crate) mod programstore;
        pub(crate) mod layer;
        pub(crate) mod shapeprogram;
        pub(super) mod patina;
    }

    pub(crate) mod util {
        pub(super) mod iterators;
        pub(crate) mod arrayutil;
    }
}

mod stage {
    pub(crate) mod axis;
    pub(crate) mod stage;
}

mod train {
    mod glcarriage;
    mod gltrain;
    mod gltrainset;

    pub(crate) use self::gltrainset::GlTrainSet;
}

mod util {
    pub(crate) mod ajax;
    pub(crate) mod enummap;
    pub(crate) mod error;
    pub(crate) mod evictlist;
    pub(crate) mod message;
    pub(crate) mod monostable;
    pub(crate) mod resizeobserver;
    #[macro_use]
    pub(crate) mod misc;
    pub(crate) mod needed;

    #[cfg(blackbox)]
    pub(crate) mod pgblackbox;

    pub use self::ajax::PgAjax;
    pub use self::message::Message;
}

mod webgl {
    pub(crate) mod canvas {
        pub(crate) mod bindery;
        pub(crate) mod canvasstore;
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
    pub(crate) use canvas::bindery::{ TextureBindery };
    pub(crate) use canvas::drawingflats::{ DrawingAllFlats, DrawingAllFlatsBuilder };
    pub(crate) use canvas::flatplotallocator::{ FlatPositionCampaignHandle };


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
        pub(crate) mod conditional;
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
    pub(crate) use program::program::{ Program, ProgramBuilder };
    pub(crate) use program::process::{ ProcessBuilder, Process };
    pub(crate) use program::compiler::make_program;
    pub(crate) use program::header::Header;
    pub(crate) use program::uniform::{ UniformProto, UniformHandle };
    pub(crate) use program::attribute::{ Attribute, AttribHandle, AttributeProto };
    pub(crate) use program::varying::Varying;
    pub(crate) use program::session::DrawingSession;
    pub(crate) use program::source::{ SourceInstrs };
    pub(crate) use program::conditional::{ Conditional, SetFlag };
    pub(crate) use program::statement::{ Statement, Declaration };
    pub(crate) use program::texture::{ Texture, TextureProto };

    pub(crate) mod global;
    mod util;
}

pub use crate::run::{ PeregrineInnerAPI, PeregrineDom, PeregrineAPI, PeregrineConfig };
pub use self::util::{ PgAjax, Message };
pub use crate::integration::PgCommanderWeb;
