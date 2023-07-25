pub type Notify<Event> = Box<dyn Fn(Event)>;
