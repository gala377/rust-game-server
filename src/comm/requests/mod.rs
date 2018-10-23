
macro_rules! req_builder {
    ($func:block) => {
        Box::new(|raw: RequestRaw| $func)
    };
}

pub mod hello {

    use std::any::Any;

    use super::super::{
        Request,
        RequestRaw,
        RequestId,
    };
    use super::super::factory::{
        Registerable,
        BoxedReqBuilder,
    };
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
            req_builder!({
                Some(Box::new(Req{})) 
            })
        }
    }
}
