use std::rc::Rc;

use super::{MutationNotify, MutationState, Notify};

pub fn mutation() -> (Rc<Notify>, MutationState, Rc<MutationNotify>) {
    vault_store::test_helpers::mutation()
}
