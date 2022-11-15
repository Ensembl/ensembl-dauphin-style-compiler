pub mod eachorevery {
    pub mod eoestruct {
        mod eoestructdata;

        #[cfg(test)]
        mod test {
            mod eoestructtest;
        }
        
        mod structbuilt;
        mod build;
        mod eoetruthy;
        mod eoestruct;
        mod expand;
        mod eoejson;
        mod replace;
        mod structtemplate; 
        mod structvalue;

        #[cfg(debug_assertions)]
        mod eoedebug;
        
        pub use expand::{ struct_select };
        pub use eoestruct::{ StructVarGroup, StructConst, StructError, struct_error_to_string };
        pub use eoejson::{ struct_to_json, struct_from_json, select_to_json };
        pub use structbuilt::{ StructBuilt };
        pub use structtemplate::{ StructTemplate, StructVar, StructPair };
        pub use structvalue::{ StructValue };
    }

    mod eoefilter;
    mod eachorevery;

    pub use eachorevery::{ EachOrEvery, EachOrEveryGroupCompatible };
    pub use eoefilter::{ EachOrEveryFilter, EachOrEveryFilterBuilder };
}

pub mod js {
    pub mod exception;
    pub mod jstojsonvalue;
    pub mod dommanip;
    pub mod timer;
    pub mod raf;
}

pub mod plumbing {
    pub mod distributor;
    pub mod onchange;
    pub mod oneshot;
}

pub mod puzzle {
    mod answer;
    mod compose;
    mod commute;
    mod constant;
    mod delayed;
    mod memoized;
    mod short;
    mod value;
    mod store;
    mod unknown;
    mod variable;

    pub use answer::{ Answer, StaticAnswer, AnswerAllocator };
    pub use commute::{ commute, commute_rc, commute_clonable, DelayedCommuteBuilder, build_commute };
    pub use compose::{ derived, compose, compose_slice, compose_slice_vec };
    pub use constant::{ constant, cache_constant, cache_constant_rc, cache_constant_clonable };
    pub use delayed::{ DelayedSetter, delayed, promise_delayed };
    pub use memoized::{ short_memoized, short_memoized_rc, short_memoized_clonable };
    pub use value::{ Value, StaticValue };
    pub use unknown::{ 
        UnknownSetter, StaticUnknownSetter, unknown, short_unknown, short_unknown_promise_clonable, 
        short_unknown_function, short_unknown_function_promise, short_unknown_clonable
    };
    pub use variable::{ variable };

    #[cfg(debug_assertions)]
    pub use compose::{ derived_debug };
}

pub mod approxnumber;
pub mod boom;
pub mod diffset;
pub mod serdetools;
pub mod console;
pub mod error;
#[macro_use]
pub mod identitynumber;
#[macro_use]
pub mod itertools;
#[macro_use]
pub mod lang;
pub mod lesqlite2;
pub mod refs;
pub mod sample;
pub mod time;
pub mod url;
pub mod skyline;

pub use approxnumber::ApproxNumber;