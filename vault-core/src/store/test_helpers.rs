use std::rc::Rc;

use super::{MutationNotify, MutationState, Notify};

pub fn mutation() -> (Rc<Notify>, MutationState, Rc<MutationNotify>) {
    let notify: Rc<Notify> = Rc::new(Box::new(|_| {}));
    let mutation_state = MutationState::default();
    let mutation_notify: Rc<MutationNotify> = Rc::new(Box::new(move |_, _, _| {}));

    (notify, mutation_state, mutation_notify)
}
