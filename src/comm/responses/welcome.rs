use std::any::Any;

use super::super::{
    Response,
    Request,
    ResponseId,
    RequestId,
    Payload,
};
use super::super::request_dispatcher::{
    Registerable,
    BoxedReqHandler,
};
use super::super::requests::hello;
use super::super::helpers;

pub struct Resp;

impl Response for Resp {

    fn id(&self) -> ResponseId {
        1
    }

    fn payload(&self) -> Payload {
        Vec::new()
    } 

}

impl Registerable for Resp {

    fn request_id() -> RequestId {
        0
    }

    fn request_handler() -> BoxedReqHandler {
        Box::new(|req: Box<dyn Request>| {
            let req: Option<&hello::Req> = helpers::cast::as_ref(&req);
            match req {
                None => {
                    println!("Illformed request in response handler!");
                    None
                },
                Some(val) => Some(Box::new(Resp{})),
            }
        })
    }

}

