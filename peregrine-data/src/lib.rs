mod allotment {
    pub(crate) mod boxes {
        pub(crate) mod boxtraits;
        pub(crate) mod leaf;
        pub(crate) mod overlay;
        pub(crate) mod stacker;
        pub(crate) mod bumper;
        pub(crate) mod container;
        pub(crate) mod root;
    }

    pub(crate) mod collision {
        mod bumppart;
        pub(crate) mod bumpprocess;
        pub(crate) mod collisionalgorithm2;
    }

    pub(crate) mod style {
        pub(crate) mod allotmentname;
        pub(crate) mod holder;
        pub(crate) mod style;
        pub(crate) mod stylebuilder;
    }

    pub(crate) mod core {
        pub(crate) mod coordsystem;
        pub(crate) mod carriageoutput;
        pub(crate) mod leaflist;
        pub(crate) mod leafrequest;
        pub(crate) mod trainstate;
    }

    pub(crate) mod globals {
        pub(crate) mod globalvalue;
        pub(crate) mod aligner;
        pub(crate) mod allotmentmetadata;
        pub(crate) mod heighttracker;
        pub(crate) mod bumping;
        pub(crate) mod trainpersistent;
        pub mod playingfield;
    }

    pub(crate) mod stylespec {
        pub(crate) mod stylegroup;
        pub(crate) mod styletree;
        pub(crate) mod styletreebuilder;
    }

    pub(crate) mod transformers {
        pub(crate) mod drawinginfo;
        pub(crate) mod transformertraits;
        pub(crate) mod simple;
        pub(crate) mod transformers;
    }

    pub(crate) mod util {
        pub(crate) mod bppxconverter;
        pub(crate) mod rangeused;
    }
}

mod api {
    mod api;
    mod agentstore;
    mod pgcore;
    mod queue;

    pub use agentstore::AgentStore;
    pub use api::{ PeregrineIntegration, CarriageSpeed };
    pub use self::pgcore::{ PeregrineCore, MessageSender, PeregrineCoreBase };
    pub use queue::{ ApiMessage, PeregrineApiQueue };
}

mod core {
    pub(crate) mod asset;
    mod config;
    mod layout;
    pub(crate) mod pixelsize;
    pub(crate) mod programbundle;
    mod scale;
    pub mod stick;
    pub(crate) mod version;
    mod viewport;
    pub(crate) mod channel;
    pub(crate) mod data;

    pub use self::config::{ PgdPeregrineConfig, ConfigKey };
    pub use self::layout::Layout;
    pub use self::scale::Scale;
    pub use stick::{ StickId, Stick, StickTopology };
    pub use self::asset::{ Asset, Assets };
    pub use viewport::Viewport;
}

mod index {
    pub(crate) mod stickstore;
    pub(crate) mod stickauthority;
    pub(crate) mod stickauthoritystore;
    pub(crate) mod jumpstore;
    pub use self::stickstore::StickStore;
    pub use self::stickauthoritystore::AuthorityStore;
}

mod shapeload {
    pub(crate) mod carriageprocess;
    mod datastore;
    mod shaperequest;
    pub(crate) mod loadshapes;
    pub(crate) mod programloader;
    pub(crate) mod programregion;
    mod resultstore;
    pub(crate) mod programdata;
    pub(crate) mod programname;

    pub use self::datastore::DataStore;
    pub use self::shaperequest::{ Region, ShapeRequest, ShapeRequestGroup };
    pub use self::programdata::ProgramData;
    pub use self::programname::ProgramName;
    pub use self::programregion::{ ProgramRegion, ProgramRegionBuilder };
    pub use self::resultstore::{ ShapeStore };
}

mod metric {
    pub(crate) mod datastreammetric;
    pub(crate) mod errormetric;
    pub(crate) mod generalreporter;
    pub(crate) mod metricreporter;
    pub(crate) mod metricutil;
    pub(crate) mod programrunmetric;
}

mod request {
    pub(crate) mod core {
        pub(crate) mod backend;
        pub(crate) mod backoff;
        pub(crate) mod manager;
        pub(crate) mod packet;
        pub(crate) mod queue;
        pub(crate) mod request;
        pub(crate) mod response;
    }

    pub(crate) mod messages {
        pub(crate) mod authorityreq;
        pub(crate) mod authorityres;
        pub(crate) mod bootstrapreq;
        pub(crate) mod bootstrapres;
        pub(crate) mod datareq;
        pub(crate) mod datares;
        pub(crate) mod failureres;
        pub(crate) mod jumpreq;
        pub(crate) mod jumpres;
        pub(crate) mod metricreq;
        pub(crate) mod programreq;
        pub(crate) mod programres;
        pub(crate) mod stickreq;
        pub(crate) mod stickres;
    }
}

mod run {
    pub mod instancepayload;
    pub mod bootstrap;
    pub mod pgcommander;
    pub mod pgdauphin;
    pub use self::pgcommander::Commander;
    pub use self::pgcommander::{ PgCommander, PgCommanderTaskSpec, add_task, complete_task, async_complete_task };
    pub use self::pgdauphin::{ PgDauphin, PgDauphinTaskSpec };
    pub use self::instancepayload::InstancePayload;
}

mod shape {
    mod carriageshapes;
    mod core;
    mod imageshape;
    pub mod rectangleshape;
    mod textshape;
    pub(crate) mod shape;
    mod programshapes;
    mod zmenu;
    mod zmenufixed;
    mod wiggleshape;

    pub use self::core::{ 
        Patina, Pen, Colour, DirectColour, Plotter, DrawnType, Hotspot
    };
    pub use self::shape::{ ShapeDemerge, Shape };
    pub use self::zmenu::ZMenu;
    pub use self::carriageshapes::CarriageShapesBuilder;
    pub use self::programshapes::ProgramShapesBuilder;
    pub use self::zmenufixed::{ ZMenuFixed, ZMenuFixedSequence, ZMenuFixedBlock, ZMenuFixedItem, ZMenuGenerator, ZMenuProxy, zmenu_fixed_vec_to_json, zmenu_to_json };
}

pub(crate) mod spacebase {
    pub mod reactive;
    pub(crate) mod spacebase;
    pub(crate) mod spacebasearea;

    pub use self::spacebase::{ SpaceBase, PartialSpaceBase, SpaceBasePoint, SpaceBasePointRef };
    pub use self::spacebasearea::{ SpaceBaseArea, HollowEdge2 };
}

pub(crate) mod switch {
    pub(crate) mod track;
    pub(crate) mod switch;
    pub(crate) mod trackconfig;
    pub(crate) mod trackconfiglist;
}

mod train {
    mod anticipate;
    pub mod drawingcarriage;
    mod carriageextent;
    mod railwaydatatasks;
    mod carriageset;
    pub(crate) mod graphics;
    mod railway;
    mod railwaydependents;
    pub(crate) mod slider;
    mod switcher;
    pub(crate) mod trainextent;
    pub(crate) mod train;
    mod trainset;

    pub use carriageextent::CarriageExtent;
    pub use drawingcarriage::{ DrawingCarriage2 };
    pub use trainextent::TrainExtent;
    pub use railway::Railway;
}

mod util {
    pub mod builder;
    pub mod lrucache;
    pub mod memoized;
    pub mod message;
    pub mod miscpromises;
    pub mod eachorevery;
    pub mod vecutils;

    pub use self::builder::Builder;
    pub use self::miscpromises::CountingPromise;
    pub use self::message::DataMessage;
    pub use self::eachorevery::{ EachOrEvery, EachOrEveryFilterBuilder };
}

pub use self::allotment::core::leafrequest::LeafRequest;
pub use self::allotment::style::style::LeafCommonStyle;
pub use self::allotment::globals::{ allotmentmetadata::GlobalAllotmentMetadata, playingfield::PlayingField };
pub use self::api::{ PeregrineCore, PeregrineCoreBase, PeregrineIntegration, PeregrineApiQueue, CarriageSpeed, AgentStore };
pub use self::core::{ Asset, Assets, PgdPeregrineConfig, ConfigKey, Stick, StickId, StickTopology, Scale, Viewport };
pub use self::core::channel::{ Channel, PacketPriority, ChannelLocation, ChannelIntegration };
pub use self::index::{ StickStore, AuthorityStore };
pub use self::shapeload::{ Region, ProgramName, ProgramRegion, ShapeStore, DataStore, ProgramData, ProgramRegionBuilder, ShapeRequest, ShapeRequestGroup };
pub use self::run::{ PgCommander, PgCommanderTaskSpec, PgDauphin, Commander, InstancePayload, add_task, complete_task, async_complete_task };
pub use self::request::core::packet::{ RequestPacket, ResponsePacket };
pub use self::request::core::backend::{ AllBackends, Backend };
pub use self::shape::{ 
    Patina, Colour, DirectColour, DrawnType, Shape, Hotspot,
    ZMenu, Pen, Plotter, ZMenuFixed, ZMenuFixedSequence, ZMenuFixedBlock, ZMenuFixedItem, ZMenuGenerator,
    ZMenuProxy, zmenu_fixed_vec_to_json, ShapeDemerge, zmenu_to_json,
    ProgramShapesBuilder, CarriageShapesBuilder
};
pub use self::allotment::core::coordsystem::{ CoordinateSystem, CoordinateSystemVariety };
pub use self::switch::switch::{ Switches };
pub use self::switch::track::Track;
pub use self::train::{ CarriageExtent, DrawingCarriage2, TrainExtent };
pub use self::util::{ CountingPromise, DataMessage, Builder };
pub use self::util::vecutils::expand_by_repeating;
pub use self::util::eachorevery::{ EachOrEvery, EachOrEveryFilterBuilder };
pub use self::spacebase::{ 
    reactive, HollowEdge2, SpaceBase, SpaceBaseArea, PartialSpaceBase,
    SpaceBasePoint, SpaceBasePointRef
};
pub use self::shape::rectangleshape::RectangleShape;
pub use self::request::core::manager::RequestManager;
