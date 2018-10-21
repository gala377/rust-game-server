
use std::any::Any;
use super::factory::*;


pub struct Hello;

impl Request for Hello {

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