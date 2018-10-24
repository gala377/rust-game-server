use std::collections::HashMap;

use super::{
    Response,
    Request,
    RequestId,
};
use super::responses;

macro_rules! register {
    ($rd:ident, $id:expr, $func:block, $msg:expr) => {
        if !$rd.register($id, Box::new(|req: Box<dyn Request>| $func)) {
            panic!($msg);
        }
    };

    ($rd:ident, $resp:ident) => {
        if !$rd.register(responses::$resp::Resp::request_id(), responses::$resp::Resp::request_handler()) {
            panic!(concat!(
                "could not register ",
                stringify!($req),
                " builder function"
            ));
        }
    };
}

pub trait RequestHandler: Fn(Box<dyn Request>) -> Option<Box<dyn Response>> {}
impl<T> RequestHandler for T where T: Fn(Box<dyn Request>) -> Option<Box<dyn Response>> {}

pub type BoxedReqHandler = Box<dyn RequestHandler<Output=Option<Box<dyn Response>>>>;

pub trait Registerable {
    fn request_id() -> RequestId;
    fn request_handler() -> BoxedReqHandler;
}


pub struct RequestDispatcher {
    pub handlers: HashMap<RequestId, BoxedReqHandler>,
}

impl RequestDispatcher {

    fn new() -> RequestDispatcher {
        let mut rd = RequestDispatcher{
            handlers: HashMap::new(),
        };

        register!(rd, welcome);

        rd 
    }

    // todo change to Result<(), RegisterError>
    pub fn register(&mut self, id: RequestId, builder: BoxedReqHandler) -> bool {
        if self.handlers.contains_key(&id) {
            false
        } else {
            self.handlers.insert(id, builder);
            true
        }
    }

    // todo change to Result<(), OverrideError>
    #[allow(dead_code)]
    pub fn overregister(&mut self, id: RequestId, builder: BoxedReqHandler) -> bool {
        if self.handlers.contains_key(&id) {
            self.handlers.insert(id, builder);
            true
        } else {
            false
        }
    }

    /// Registers builder if none exists. Overregisters otherwise.
    #[allow(dead_code)]
    pub fn force_register(&mut self, id: RequestId, builder: BoxedReqHandler) {
        if self.handlers.contains_key(&id) {
            self.overregister(id, builder);
        } else {
            self.register(id, builder);
        }
    }

}