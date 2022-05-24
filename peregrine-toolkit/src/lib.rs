pub mod js {
    pub mod exception;
}

pub mod plumbing {
    pub mod distributor;
    pub mod onchange;
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
    pub use commute::{ commute, commute_arc, commute_clonable, DelayedCommuteBuilder, build_commute };
    pub use compose::{ derived, compose, compose_slice };
    pub use constant::{ constant, cache_constant, cache_constant_arc, cache_constant_clonable };
    pub use delayed::{ DelayedSetter, delayed, promise_delayed };
    pub use memoized::{ short_memoized, short_memoized_arc, short_memoized_clonable };
    pub use value::{ Value, StaticValue };
    pub use unknown::{ 
        UnknownSetter, StaticUnknownSetter, unknown, short_unknown, short_unknown_promise_clonable, 
        short_unknown_function, short_unknown_function_promise, short_unknown_clonable
    };
    pub use variable::{ variable };

    #[cfg(debug_assertions)]
    pub use compose::{ derived_debug };
}

pub mod boom;
pub mod cbor;
pub mod console;
pub mod refs;
pub mod sample;
pub mod time;
pub mod url;
pub mod skyline;
