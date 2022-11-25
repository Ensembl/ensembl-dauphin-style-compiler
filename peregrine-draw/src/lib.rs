mod domcss {
    pub(crate) mod dom;
    pub(crate) mod size;
    mod shutdown;
    mod yposdetector;
}

mod input {
    mod core {
        pub mod input;
    }

    pub(crate) mod low {
        pub(crate) mod gesture {
            pub(crate) mod core {
                pub(crate) mod cursor;
                pub(crate) mod finger;
                pub(crate) mod gesture;
                pub(crate) mod gesturenode;
                pub(super) mod transition;    
            }

            pub(crate) mod node {
                mod commontools;
                pub(crate) mod pinch;
                pub(crate) mod drag;
                pub(super) mod marquee;
                pub(crate) mod unknown;    
            }
        }

        pub(super) mod pointer;
        pub(crate) mod event;
        pub(crate) mod keyboardinput;
        pub(crate) mod mouseinput;
        pub(crate) mod lowlevel; 
        pub(crate) mod keyspec;
        pub(crate) mod modifiers;
        pub mod mapping;
    }

    mod regimes {
        pub(crate) mod regime;
        mod gotoregime;
        mod dragregime;
        mod setregime;
    }

    pub(crate) mod translate {
        pub(super) mod measure;
        pub(crate) mod animqueue;
        pub(crate) mod axisphysics;
        pub(crate) mod translateinput;
        pub(crate) mod debug;
        pub(crate) mod targetreporter;
        pub(crate) mod translatehotspots;

        pub use self::translateinput::InputTranslator;
    }

    pub use self::core::input::{ Input, InputEvent, InputEventKind };
}

mod integration {
    mod bell;
    pub(crate) mod pgcommander;
    pub(crate) mod pgdauphin;
    pub(crate) mod pgintegration;
    mod custom;
    mod stream;

    pub use self::pgcommander::PgCommanderWeb;
}

mod run {
    mod buildconfig;
    pub mod api;
    mod config;
    mod globalconfig;
    pub mod inner;
    mod frame;
    mod mousemove;
    pub(crate) mod report;
    pub(crate) mod sound;

    pub use self::config::{ PgPeregrineConfig, PgConfigKey, CursorCircumstance };
    pub use self::globalconfig::PeregrineConfig;
    pub use self::api::{ PeregrineAPI };
    pub use self::inner::{ PeregrineInnerAPI };
}

mod shape {
    pub(crate) mod core {
        pub(crate) mod prepareshape;
        pub(crate) mod drawshape;
        pub(super) mod directcolourdraw;
        pub(crate) mod texture;
        pub(crate) mod wigglegeometry;
    }

    pub(crate) mod spectres {
        pub(crate) mod ants;
        pub(crate) mod stain;
        pub(crate) mod spectre;
        pub(crate) mod spectraldrawing;
        pub(crate) mod spectremanager;
    }

    pub(crate) mod triangles {
        pub(crate) mod drawgroup;
        pub(crate) mod triangleadder;
        pub(crate) mod rectangles;
    }

    pub(crate) mod canvasitem {
        pub(super) mod bardots;
        pub(crate) mod heraldry;
        pub(crate) mod bitmap;
        pub(crate) mod text;
        pub(crate) mod imagecache;
        pub(crate) mod structuredtext;
    }

    pub(crate) mod layers {
        pub(crate) mod drawing;
        pub(crate) mod drawingtools;
        pub(crate) mod consts;
        pub(crate) mod geometry;
        pub(crate) mod programstore;
        pub(crate) mod layer;
        pub(crate) mod shapeprogram;
        pub(super) mod patina;
    }

    pub(crate) mod util {
        pub(super) mod eoethrow;
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

    pub(crate) use self::gltrainset::GlRailway;
}

mod util {
    pub(crate) mod error;
    pub(crate) mod message;
    pub(crate) mod monostable;
    pub(crate) mod debounce;
    pub(crate) mod resizeobserver;
    pub(crate) mod fonts;
    #[macro_use]
    pub(crate) mod misc;
    pub use self::message::{ Message, Endstop };
}

mod webgl {
    pub(crate) mod canvas {
        pub(crate) mod composition {
            pub(crate) mod areabuilder;
            pub(crate) mod packer;
            pub(crate) mod compositionbuilder;
            pub(crate) mod canvasitem;
        }

        pub(crate) mod binding {
            pub(crate) mod binding;
            pub(crate) mod texturebinding;
            pub(crate) mod weave;
        }

        pub(crate) mod htmlcanvas {
            pub(crate) mod canvassource;
            pub(crate) mod scratchcanvases;
            pub(crate) mod canvasinuse;
        }
    }

    pub(super) mod gpuspec {
        pub(crate) mod glarity;
        pub(crate) mod gpuspec;
        pub(crate) mod precision;
        mod glsize;
    }

    pub(crate) use gpuspec::gpuspec::{ GPUSpec, Phase };
    pub(crate) use gpuspec::glarity::GLArity;
    pub(crate) use gpuspec::precision::Precision;

    pub(crate) mod program {
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
    pub(crate) use program::program::{ ProgramBuilder };
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
    pub(crate) use program::texture::{ TextureProto };

    pub(crate) mod global;
    pub(crate) mod glbufferstore;
    mod util;
}

mod hotspots {
    mod coordconverter;
    mod trackinghotspots;
    mod windowhotspots;
    mod drawhotspotstore;
    pub(crate) mod drawinghotspots;
}

pub use crate::run::{ PeregrineInnerAPI, PeregrineAPI, PeregrineConfig };
pub use self::util::{ Message, Endstop };
pub use crate::integration::PgCommanderWeb;
