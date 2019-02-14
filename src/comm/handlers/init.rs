use crate::comm::handlers::{concrete, DefaultBuilder, Dispatcher};

macro_rules! register {
    ($f:ident, $h:ident) => {
        if !$f.register(
            concrete::$h::Handler::req_id(),
            concrete::$h::Handler::build_handler(),
        ) {
            panic!(concat!(
                "could not register ",
                stringify!($h),
                " builder function"
            ));
        }
    };
}

/// Returns RequestFactory struct with
/// all the factory functions registered.
pub fn new_dispatcher() -> Dispatcher {
    let mut f = Dispatcher::new();

    register!(f, hello);
    register!(f, register_player); 
    register!(f, add_unit);

    f
}
