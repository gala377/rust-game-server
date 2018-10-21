use std::any::Any;
use super::factory::{
    BoxedReqBuilder,
    RequestRaw,
    Request,
};


pub struct Cast;

impl Cast {

    pub fn as_mut_ref<T: Any>(obj: &mut Box<dyn Request>) -> Option<&mut T> {
        obj.as_any_mut().downcast_mut::<T>()
    }

    pub fn as_ref<T: Any>(obj: &Box<dyn Request>) -> Option<&T> {
        obj.as_any().downcast_ref::<T>()
    }
}


// pub fn ResponseBuilderWrapper<T>(
//     builder: Box<dyn Fn(T) -> Box<Response>>) -> BoxedRespBuilder {
//
//    return  move |raw: Box<dyn Request>| {
//          let casted = Cast<T>::as_ref();
//          builder(casted) 
//    }
// }