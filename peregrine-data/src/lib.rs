mod allotment {
    pub(crate) mod containers {
        pub(crate) mod overlay;
        pub(crate) mod stacker;
        pub(crate) mod bumper;
        pub(crate) mod container;
        pub(crate) mod haskids;
        pub(crate) mod root;
    }

    pub(crate) mod leafs {
        pub(crate) mod floating;
        pub(crate) mod anchored;
        pub(crate) mod auxleaf;
        pub(crate) mod leafrequest;
    }

    pub(crate) mod layout {
        pub(crate) mod contentsize;
        pub(crate) mod layouttree;
        pub(crate) mod layoutcontext;
        pub(crate) mod leafrequestsize;
    }
    
    pub(crate) mod collision {
        pub(crate) mod bumprequest;
        mod bumppart;
        pub(crate) mod bumpprocess;
        mod standardalgorithm;
        pub(crate) mod algorithmbuilder;
    }

    pub(crate) mod style {
        pub(crate) mod containerstyle;
        pub(crate) mod leafstyle;
        pub(crate) mod metadataproperty;
        pub(super) mod pathtree;
        pub(crate) mod styletree;
    }

    pub(crate) mod core {
        pub(crate) mod allotmentname;
        pub(crate) mod floatingcarriage;
        pub(crate) mod leafrequestsource;
        pub(crate) mod rangeused;
    }
}

pub(crate) mod globals {
    pub(crate) mod correlate;
    pub(crate) mod globalvalue;
    pub(crate) mod aligner;
    pub(crate) mod allotmentmetadata;
    pub(crate) mod heighttracker;
    pub(crate) mod bumping;
    pub(crate) mod trainpersistent;
    pub mod playingfield;
    pub(crate) mod trainstate;
}

mod api {
    pub(crate) mod api;
    mod agentstore;
    mod pgcore;
    mod queue;
    mod instanceinfo;

    pub use agentstore::AgentStore;
    pub use api::{ PeregrineIntegration, CarriageSpeed, TrainIdentity };
    pub use self::pgcore::{ PeregrineCore, MessageSender, PeregrineCoreBase };
    pub use queue::{ PeregrineApiQueue };
    pub use instanceinfo::InstanceInformation;
}

mod core {
    pub(crate) mod channel {
        pub(crate) mod backendnamespace;
        pub(crate) mod accessorresolver;
        pub(crate) mod channelintegration;
        pub(crate) mod channelregistry;
        pub(crate) mod channelboot;
        pub(crate) mod wrappedchannelsender;
    }

    pub(crate) mod program {
        mod packedprogramspec;
        pub(crate) mod programspec;
        pub(crate) mod programbundle;
    }

    pub(crate) mod dataalgorithm;
    pub(crate) mod asset;
    mod config;
    mod layout;
    pub(crate) mod pixelsize;
    mod scale;
    pub mod stick;
    pub(crate) mod tagpred;
    pub(crate) mod version;
    mod viewport;
    pub(crate) mod data;
    pub(crate) mod coordsystem;

    pub use self::config::{ PgdPeregrineConfig, ConfigKey };
    pub use self::layout::Layout;
    pub use self::scale::Scale;
    pub use self::program::programspec::{ ProgramModel, ProgramSetting };
    pub use stick::{ StickId, Stick, StickTopology };
    pub use self::asset::{ Asset, Assets };
    pub use viewport::Viewport;
}

mod index {
    pub(crate) mod stickstore;
    pub(crate) mod smallvaluesstore;
    pub(crate) mod jumpstore;
    pub use self::stickstore::StickStore;
    pub use self::smallvaluesstore::SmallValuesStore;
}

mod shapeload {
    pub(crate) mod anticipate;
    pub(crate) mod carriagebuilder;
    mod datastore;
    mod shaperequest;
    pub(crate) mod shaperequestgroup;
    pub(crate) mod loadshapes;
    mod resultstore;
    pub(crate) mod programname;

    pub use self::datastore::{ DataStore };
    pub use self::shaperequest::{ Region, ShapeRequest };
    pub use self::loadshapes::LoadMode;
    pub use self::resultstore::{ ShapeStore, RunReport };
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
        pub(crate) mod maxirequest;
        pub(crate) mod maxiresponse;
        pub(crate) mod packet;
        pub(crate) mod queue;
        pub(crate) mod minirequest;
        pub(crate) mod miniresponse;
        mod pendingattemptqueue;
        mod attemptmatch;
        pub(crate) mod sidecars;
        pub(crate) mod packetpriority;
        mod trafficcontrol;
    }

    pub(crate) mod minirequests {
        pub(crate) mod bootchannelreq;
        pub(crate) mod bootchannelres;
        pub(crate) mod datareq;
        pub(crate) mod datares;
        pub(crate) mod expandreq;
        pub(crate) mod expandres;
        pub(crate) mod failureres;
        pub(crate) mod jumpreq;
        pub(crate) mod jumpres;
        pub(crate) mod metricreq;
        pub(crate) mod programreq;
        pub(crate) mod programres;
        pub(crate) mod smallvaluesreq;
        pub(crate) mod smallvaluesres;
        pub(crate) mod stickreq;
        pub(crate) mod stickres;
    }

    pub(crate) mod tracks {
        pub(crate) mod expansionmodel;
        mod switchtree;
        pub(crate) mod packedtrackres;
        pub(crate) mod trackdata;
        pub(crate) mod trackmodel;
        pub(crate) mod trackres;
    }
}

mod run {
    pub mod pgcommander;
    pub mod pgdauphin;
    pub use self::pgcommander::Commander;
    pub use self::pgcommander::{ PgCommander, PgCommanderTaskSpec, add_task, complete_task, async_complete_task };
    pub use self::pgdauphin::{ PgDauphin };
}

mod shape {
    pub(crate) mod requestedshapescontainer;
    pub(crate) mod originstats;
    mod core;
    pub mod emptyshape;
    mod imageshape;
    pub(crate) mod metadata;
    pub mod rectangleshape;
    pub mod polygonshape;
    pub(crate) mod textshape;
    pub(crate) mod shape;
    mod programshapes;
    mod settingmode;
    mod wiggleshape;

    pub use self::core::{ 
        Patina, Pen, Colour, DirectColour, Plotter, DrawnType, HotspotPatina, PenGeometry,
        AttachmentPoint
    };
    pub use self::settingmode::SettingMode;
    pub use self::shape::{ ShapeDemerge, Shape };
    pub use self::requestedshapescontainer::RequestedShapesContainer;
    pub use self::programshapes::ProgramShapesBuilder;
}

pub(crate) mod spacebase {
    pub mod reactive;
    pub(crate) mod spacebase;
    pub(crate) mod spacebasearea;

    pub use self::spacebase::{ SpaceBase, PartialSpaceBase, SpaceBasePoint, SpaceBasePointRef };
    pub use self::spacebasearea::{ SpaceBaseArea, HollowEdge2 };
}

pub(crate) mod switch {
    pub(crate) mod expansion;
    pub(crate) mod track;
    pub(crate) mod switch;
    pub(crate) mod switches;
    pub(crate) mod trackconfig;
    pub(crate) mod trackconfiglist;
}

mod train {
    mod core {
        pub(crate) mod party;
        pub(crate) mod switcher;    
    }

    pub(crate) mod drawing {
        pub mod drawingcarriage;
        pub(crate) mod drawingtrain;
        pub(crate) mod drawingtrainset;
    }

    pub(crate) mod main {
        pub(crate) mod abstracttrain;
        pub(crate) mod datatasks;
        pub(crate) mod railway;
        pub(crate) mod train;    
    }

    pub mod model {
        pub(crate) mod carriageextent;
        pub(crate) mod trainextent;
    }

    pub(crate) mod graphics;

    pub use model::carriageextent::{ CarriageExtent };
    pub use drawing::drawingcarriage::{ DrawingCarriage };
}

mod util {
    pub mod builder;
    pub mod lrucache;
    pub mod memoized;
    pub mod message;
    pub mod miscpromises;
    pub mod vecutils;

    pub use self::builder::Builder;
    pub use self::miscpromises::CountingPromise;
    pub use self::message::DataMessage;
}

pub(crate) mod hotspots {
    pub(crate) mod hotspots;
}

pub use self::allotment::leafs::leafrequest::LeafRequest;
pub use self::allotment::leafs::auxleaf::AuxLeaf;
pub use self::globals::{ allotmentmetadata::GlobalAllotmentMetadata, playingfield::PlayingField };
pub use self::api::{ PeregrineCore, PeregrineCoreBase, PeregrineIntegration, PeregrineApiQueue, TrainIdentity, CarriageSpeed, AgentStore, InstanceInformation };
pub use self::core::{ Asset, Assets, PgdPeregrineConfig, ConfigKey, Stick, StickId, StickTopology, Scale, Viewport, ProgramModel, ProgramSetting };
pub use self::core::channel::accessorresolver::{ AccessorResolver };
pub use self::core::channel::backendnamespace::BackendNamespace;
pub use self::core::dataalgorithm::DataAlgorithm;
pub use self::core::channel::channelintegration::{ ChannelIntegration, ChannelSender, ChannelResponse, TrivialChannelResponse, ChannelMessageDecoder, null_payload };
pub use self::index::{ StickStore, SmallValuesStore };
pub use self::core::program::programbundle::{ SuppliedBundle, UnpackedSuppliedBundle };
pub use self::shapeload::{ Region, ShapeStore, DataStore, ShapeRequest, LoadMode, RunReport };
pub use self::run::{ PgCommander, PgCommanderTaskSpec, PgDauphin, Commander, add_task, complete_task, async_complete_task };
pub use self::request::core::maxirequest::{ MaxiRequest };
pub use self::request::core::maxiresponse::{ MaxiResponse, MaxiResponseDeserialize };
pub use self::request::core::minirequest::{ MiniRequest, MiniRequestAttempt };
pub use self::request::core::miniresponse::MiniResponse;
pub use self::request::core::packetpriority::PacketPriority;
pub use self::request::core::backend::{ AllBackends, Backend };
pub use self::shape::shape::DrawingShape;
pub use self::shape::{ 
    Patina, Colour, DirectColour, DrawnType, Shape, HotspotPatina, PenGeometry,
    Pen, Plotter, ShapeDemerge, SettingMode,
    ProgramShapesBuilder, RequestedShapesContainer, AttachmentPoint
};
pub use self::core::coordsystem::{ CoordinateSystem };
pub use self::switch::switches::{ Switches };
pub use self::switch::track::Track;
pub use self::train::{ DrawingCarriage, CarriageExtent };
pub use self::util::{ CountingPromise, DataMessage, Builder };
pub use self::util::vecutils::expand_by_repeating;
pub use self::spacebase::{ 
    reactive, HollowEdge2, SpaceBase, SpaceBaseArea, PartialSpaceBase,
    SpaceBasePoint, SpaceBasePointRef
};
pub use self::shape::polygonshape::PolygonShape;
pub use self::shape::rectangleshape::RectangleShape;
pub use self::shape::textshape::TextShape;
pub use self::core::data::ReceivedData;
pub use self::request::core::manager::RequestManager;
pub use self::request::tracks::trackmodel::{ TrackMapping, TrackModel, TrackModelDeserialize };
pub use self::request::tracks::expansionmodel::ExpansionModel;
pub use self::request::minirequests::failureres::FailureRes;
pub use self::request::minirequests::smallvaluesreq::SmallValuesReq;
pub use self::request::minirequests::smallvaluesres::SmallValuesRes;
pub use self::request::minirequests::bootchannelreq::BootChannelReq;
pub use self::request::minirequests::bootchannelres::BootChannelRes;
pub use self::request::minirequests::stickreq::StickReq;
pub use self::request::minirequests::stickres::StickRes;
pub use self::request::minirequests::jumpreq::JumpReq;
pub use self::request::minirequests::jumpres::{ JumpLocation, JumpRes };
pub use self::request::minirequests::datareq::DataRequest;
pub use self::request::minirequests::datares::{ DataRes, DataResponse };
pub use self::request::minirequests::expandreq::{ ExpandReq };
pub use self::request::minirequests::expandres::{ ExpandRes };
pub use self::request::minirequests::programreq::{ ProgramReq };
pub use self::request::minirequests::programres::{ ProgramRes };
pub use self::hotspots::hotspots::{ 
    HotspotResult, HotspotGroupEntry, SingleHotspotEntry, SpecialClick
};
