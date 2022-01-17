mod allotment {
    mod lineargroup {
        pub(crate) mod secondary;
        pub(super) mod offsetbuilder;
        pub(crate) mod lineargroup;
    }

    pub(crate) mod core {
        pub(crate) mod allotment;
        pub(crate) mod allotmentmetadata;
        pub(crate) mod allotmentrequest;
        pub(crate) mod basicallotmentspec;
        pub(crate) mod dustbinallotment;
        pub(crate) mod universe;    
    }

    mod tree {
        pub(crate) mod collisionnode;
        pub(crate) mod leafboxtransformer;
        pub(crate) mod leafboxlinearentry;    
        pub(crate) mod maintrack;
        pub(crate) mod maintrackspec;
        pub(crate) mod allotmentbox;    
    }
}

mod api {
    mod api;
    mod agentstore;
    mod pgcore;
    mod queue;

    pub use agentstore::AgentStore;
    pub use api::{ PeregrineIntegration, CarriageSpeed, PlayingField };
    pub use self::pgcore::{ PeregrineCore, MessageSender, PeregrineCoreBase };
    pub use queue::{ ApiMessage, PeregrineApiQueue };
}

mod core {
    pub(crate) mod asset;
    mod config;
    mod layout;
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

mod lane {
    mod datastore;
    mod shaperequest;
    pub(crate) mod shapeloader;
    pub(crate) mod programloader;
    pub(crate) mod programregion;
    mod resultstore;
    pub(crate) mod programdata;
    pub(crate) mod programname;

    pub use self::datastore::DataStore;
    pub use self::shaperequest::{ Region, ShapeRequest };
    pub use self::programdata::ProgramData;
    pub use self::programname::ProgramName;
    pub use self::programregion::{ ProgramRegion, ProgramRegionBuilder };
    pub use self::resultstore::{ LaneStore };
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
    mod core;
    mod imageshape;
    pub mod rectangleshape;
    mod textshape;
    pub(crate) mod shape;
    mod shapelist;
    mod zmenu;
    mod zmenufixed;
    mod wiggleshape;

    pub use self::core::{ 
        Patina, Pen, Colour, DirectColour, Plotter, DrawnType
    };
    pub use self::shape::{ Shape, ShapeDemerge, ShapeDetails, ShapeCommon };
    pub use self::zmenu::ZMenu;
    pub use self::shapelist::{ ShapeListBuilder, ShapeList };
    pub use self::zmenufixed::{ ZMenuFixed, ZMenuFixedSequence, ZMenuFixedBlock, ZMenuFixedItem, ZMenuGenerator, ZMenuProxy, zmenu_fixed_vec_to_json };
}

pub(crate) mod spacebase {
    pub(crate) mod parametric;
    pub(crate) mod spacebase;
    pub(crate) mod spacebasearea;

    pub use self::parametric::{ VariableValues, ParameterValue, Flattenable, Substitutions, Variable };
    pub use self::spacebase::{ SpaceBase, HoleySpaceBase, SpaceBaseParameterLocation, SpaceBasePointRef };
    pub use self::spacebasearea::{ SpaceBaseArea, HoleySpaceBaseArea, SpaceBaseAreaParameterLocation, HollowEdge };
}

pub(crate) mod switch {
    pub(crate) mod track;
    pub(crate) mod switch;
    pub(crate) mod trackconfig;
    pub(crate) mod trackconfiglist;
}

mod train {
    mod anticipate;
    pub(crate) mod carriage;
    mod railwayevent;
    mod carriageextent;
    mod carriageset;
    mod railway;
    mod railwaydependents;
    mod trainextent;
    mod train;
    mod trainset;

    pub use carriageextent::CarriageExtent;
    pub use carriage::{ Carriage, CarriageSerial };
    pub use train::{ Train, TrainSerial };
    pub use railway::Railway;
}

mod util {
    pub mod builder;
    pub mod lrucache;
    pub mod memoized;
    pub mod message;
    pub mod miscpromises;
    pub mod eachorevery;
    pub mod ringarray;
    pub mod vecutils;

    pub use self::builder::Builder;
    pub use self::miscpromises::CountingPromise;
    pub use self::message::DataMessage;
    pub use self::eachorevery::EachOrEvery;
}

pub use self::api::{ PeregrineCore, PeregrineCoreBase, PeregrineIntegration, PeregrineApiQueue, CarriageSpeed, AgentStore, PlayingField };
pub use self::core::{ Asset, Assets, PgdPeregrineConfig, ConfigKey, Stick, StickId, StickTopology, Scale, Viewport };
pub use self::core::channel::{ Channel, PacketPriority, ChannelLocation, ChannelIntegration };
pub use self::index::{ StickStore, AuthorityStore };
pub use self::lane::{ Region, ProgramName, ProgramRegion, LaneStore, DataStore, ProgramData, ProgramRegionBuilder, ShapeRequest };
pub use self::run::{ PgCommander, PgCommanderTaskSpec, PgDauphin, Commander, InstancePayload, add_task, complete_task, async_complete_task };
pub use self::request::core::packet::{ RequestPacket, ResponsePacket };
pub use self::request::core::backend::{ AllBackends, Backend };
pub use self::shape::{ 
    Patina, Colour, DirectColour, DrawnType, ShapeDetails, ShapeCommon,
    ZMenu, Pen, Plotter, Shape, ZMenuFixed, ZMenuFixedSequence, ZMenuFixedBlock, ZMenuFixedItem, ZMenuGenerator, ShapeListBuilder,
    ShapeList, ZMenuProxy, zmenu_fixed_vec_to_json, ShapeDemerge
};
pub use self::allotment::core::allotment::{ Allotment, CoordinateSystem };
pub use self::allotment::core::allotmentrequest::AllotmentRequest;
pub use self::allotment::core::allotmentmetadata::{
    AllotmentMetadataStore, AllotmentMetadata, AllotmentMetadataRequest, AllotmentMetadataReport, MetadataMergeStrategy
};
pub use self::allotment::core::universe::Universe;
pub use self::switch::switch::{ Switches };
pub use self::switch::track::Track;
pub use self::train::{ Carriage, CarriageExtent, Train, TrainSerial, CarriageSerial };
pub use self::util::{ CountingPromise, DataMessage, Builder };
pub use self::util::ringarray::{ DataFilter, DataFilterBuilder };
pub use self::util::vecutils::expand_by_repeating;
pub use self::util::eachorevery::EachOrEvery;
pub use self::spacebase::{ 
    SpaceBase, SpaceBaseArea, VariableValues, ParameterValue, HoleySpaceBaseArea, Flattenable, SpaceBasePointRef,
    SpaceBaseAreaParameterLocation, Substitutions, HoleySpaceBase,
    SpaceBaseParameterLocation, HollowEdge, Variable
};
pub use self::shape::rectangleshape::RectangleShape;
pub use self::request::core::manager::RequestManager;
