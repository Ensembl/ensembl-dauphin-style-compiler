mod allotment {
    pub(crate) mod allotment;
    pub(crate) mod allotmentmetadata;
    pub(crate) mod allotmentrequest;
    pub(crate) mod baseallotmentrequest;
    pub(crate) mod dustbinallotment;
    pub(crate) mod lineargroup;
    pub(crate) mod maintrack;
    pub(crate) mod offsetallotment;
    pub(crate) mod universe;
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
    mod scale;
    pub mod stick;
    mod viewport;

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
    pub use self::stickauthoritystore::StickAuthorityStore;
}

mod lane {
    mod datastore;
    mod shaperequest;
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
    pub(crate) mod metricreporter;
    pub(crate) mod metricutil;
    pub(crate) mod programrunmetric;
}

mod request {
    pub(crate) mod backoff;
    pub(crate) mod bootstrap;
    pub(crate) mod channel;
    pub(crate) mod data;
    pub(crate) mod failure;
    pub(crate) mod jump;
    pub(crate) mod manager;
    pub(crate) mod packet;
    pub(crate) mod queue;
    pub(crate) mod program;
    pub(crate) mod request;
    pub(crate) mod stick;
    pub(crate) mod stickauthority;
    pub use self::channel::{ Channel, ChannelIntegration, ChannelLocation, PacketPriority };
    pub use self::program::{ ProgramLoader };
    pub use self::manager::RequestManager;
    pub use self::stick::issue_stick_request;
    pub use self::jump::issue_jump_request;
}

mod run {
    pub mod instancepayload;
    pub mod pgcommander;
    pub mod pgdauphin;
    pub use self::pgcommander::Commander;
    pub use self::pgcommander::{ PgCommander, PgCommanderTaskSpec, add_task, complete_task, async_complete_task };
    pub use self::pgdauphin::{ PgDauphin, PgDauphinTaskSpec };
    pub use self::instancepayload::InstancePayload;
}

mod shape {
    mod core;
    pub(crate) mod shape;
    mod shapelist;
    mod zmenu;
    mod zmenufixed;

    pub use self::core::{ 
        Patina, Pen, Colour, DirectColour, Plotter, DrawnType
    };
    pub use self::shape::{ Shape, ShapeDemerge, RectangleShape };
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
    mod carriageevent;
    mod carriageset;
    mod train;
    mod trainset;

    pub use carriage::{ CarriageId, Carriage };
    pub use trainset::TrainSet;
}

mod util {
    pub mod builder;
    pub mod cbor;
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
pub use self::index::{ StickStore, StickAuthorityStore };
pub use self::lane::{ Region, ProgramName, ProgramRegion, LaneStore, DataStore, ProgramData, ProgramRegionBuilder, ShapeRequest };
pub use self::run::{ PgCommander, PgCommanderTaskSpec, PgDauphin, Commander, InstancePayload, add_task, complete_task, async_complete_task };
pub use self::request::{ Channel, ChannelIntegration, ChannelLocation, PacketPriority, ProgramLoader, RequestManager, issue_stick_request, issue_jump_request };
pub use self::shape::{ 
    Patina, Colour, DirectColour, DrawnType,
    ZMenu, Pen, Plotter, Shape, ZMenuFixed, ZMenuFixedSequence, ZMenuFixedBlock, ZMenuFixedItem, ZMenuGenerator, ShapeListBuilder,
    ShapeList, ZMenuProxy, zmenu_fixed_vec_to_json, ShapeDemerge, RectangleShape
};
pub use self::allotment::allotment::{ Allotment, CoordinateSystem };
pub use self::allotment::allotmentrequest::AllotmentRequest;
pub use self::allotment::allotmentmetadata::{
    AllotmentMetadataStore, AllotmentMetadata, AllotmentMetadataRequest, AllotmentMetadataReport
};
pub use self::allotment::universe::Universe;
pub use self::switch::switch::{ Switches };
pub use self::switch::track::Track;
pub use self::train::{ Carriage, CarriageId };
pub use self::util::cbor::{ cbor_coerce_string, cbor_bytes };
pub use self::util::{ CountingPromise, DataMessage, Builder };
pub use self::util::ringarray::{ DataFilter, DataFilterBuilder };
pub use self::util::vecutils::expand_by_repeating;
pub use self::util::eachorevery::EachOrEvery;
pub use self::spacebase::{ 
    SpaceBase, SpaceBaseArea, VariableValues, ParameterValue, HoleySpaceBaseArea, Flattenable, SpaceBasePointRef,
    SpaceBaseAreaParameterLocation, Substitutions, HoleySpaceBase,
    SpaceBaseParameterLocation, HollowEdge, Variable
};
