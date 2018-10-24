
use std::any::Any;

use super::super::factory::{BoxedReqBuilder, Registerable};
use super::super::{Request, RequestId, RequestRaw};

// Types

pub struct Req;

impl Request for Req {
    fn id(&self) -> RequestId {
        0
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Registerable for Req {
    fn id() -> RequestId {
        0
    }

    fn builder() -> BoxedReqBuilder {
        Box::new(|_raw: RequestRaw| Some(Box::new(Req {})))
    }
}
