pub mod hello {

    use std::any::Any;

    use super::super::Request;


    // Types

    pub struct Req;

    impl Request for Req {

        fn id(&self) -> u32 {
            0
        }

        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }

    }
}
