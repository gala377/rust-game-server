use super::factory::{Registerable, RequestFactory};
use super::requests;

macro_rules! register {
    ($f:ident, $id:expr, $func:block, $msg:expr) => {
        if !$f.register($id, Box::new(|raw: RequestRaw| $func)) {
            panic!($msg);
        }
    };

    ($f:ident, $req:ident) => {
        if !$f.register(requests::$req::Req::id(), requests::$req::Req::builder()) {
            panic!(concat!(
                "could not register ",
                stringify!($req),
                " builder function"
            ));
        }
    };
}

/// Returns RequestFactory struct with
/// all the factory functions registered.
pub fn init() -> RequestFactory {
    let mut f = RequestFactory::new();

    register!(f, hello);

    f
}
