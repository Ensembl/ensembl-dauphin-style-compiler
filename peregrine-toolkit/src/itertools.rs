#[macro_export]
macro_rules! lengths_match {
    ($self:expr,$first:ident,$($rest:ident),*) => {
        (|| {
            let len = $self.$first.len();
            $( if $self.$rest.len() != len { return false; } )*
            true
        })()
    }
}

#[macro_export]
macro_rules! multizip {
    ($($arg:ident),*;$cb:expr) => {
        {
            use itertools::izip;

            for ($($arg),*) in izip!($($arg.iter().cloned()),*) {
                $cb
            }
        }
    };

    ($self:expr;$($arg:ident),*;$cb:expr) => {
        {
            use itertools::izip;

            for ($($arg),*) in izip!($($self.$arg.iter().cloned()),*) {
                $cb
            }
        }
    }
}
