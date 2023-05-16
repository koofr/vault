use super::Event;

pub type Notify = Box<dyn Fn(Event)>;
