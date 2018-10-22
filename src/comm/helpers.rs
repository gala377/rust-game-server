/// Holds functions helping dynamic casting Request trait object
/// to underlying struct.
pub mod cast {

    use std::any::Any;

    use super::super::Request;


    /// Returns Some(val) where val is mutable reference to the underlying struct
    /// of the provided Request trait object. 
    /// None if cast was unsuccessful.
    #[allow(dead_code)]
    pub fn as_mut_ref<T: Any>(obj: &mut Box<dyn Request>) -> Option<&mut T> {
        obj.as_any_mut().downcast_mut::<T>()
    }

    /// Returns Some(val) where val is reference to the underlying struct
    /// of the provided Request trait object. 
    /// None if cast was unsuccessful.
    pub fn as_ref<T: Any>(obj: &Box<dyn Request>) -> Option<&T> {
        obj.as_any().downcast_ref::<T>()
    }
}

// todo 
// pub fn ResponseBuilderWrapper<T>(
//     builder: Box<dyn Fn(T) -> Box<Response>>) -> BoxedRespBuilder {
//
//    return  move |raw: Box<dyn Request>| {
//          let casted = Cast<T>::as_ref();
//          builder(casted) 
//    }
// }