use std::sync::Arc;

#[macro_export]
macro_rules! lock {
    ($x: expr) => {{
        match $x.lock() {
            Ok(v) => v,
            Err(_) => {
                panic!("ENSEMBL ERROR LOCATION {}/{}/{}",file!(),line!(),column!());
            }
        }
    }}
}

// upcast derived from MIT-licensed code by Connie Hilarides

pub trait UpcastFrom<T: ?Sized> {
    fn up_from_arc(value: Arc<T>) -> Arc<Self>;
    fn up_from(value: &T) -> &Self;
    fn up_from_mut(value: &mut T) -> &mut Self;
}

pub trait Upcast<U: ?Sized> {
    fn up_arc(value: Arc<Self>) -> Arc<U>;
    fn up(&self) -> &U;
    fn up_mut(&mut self) -> &mut U;
}

impl<T: ?Sized, U: ?Sized> Upcast<U> for T where U: UpcastFrom<T> {
    fn up_arc(value: Arc<Self>) -> Arc<U> { U::up_from_arc(value) }
    fn up(&self) -> &U { U::up_from(self) }
    fn up_mut(&mut self) -> &mut U { U::up_from_mut(self) }
}

#[macro_export]
macro_rules! upcast {
    ($sub_ty:ty,$super_ty:ty) => {
        impl UpcastFrom<$sub_ty> for $super_ty {
            fn up_from_arc(value: Arc<$sub_ty>) -> Arc<Self> { value }
            fn up_from(value: &$sub_ty) -> &Self { value }
            fn up_from_mut(value: &mut $sub_ty) -> &mut Self { value }
        }        
    };
}
