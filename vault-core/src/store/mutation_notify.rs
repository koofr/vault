use super::{MutationEvent, MutationState, State};

pub type MutationNotify = Box<dyn Fn(MutationEvent, &mut State, &mut MutationState)>;
