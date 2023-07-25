pub type MutationNotify<MutationEvent, State, MutationState> =
    Box<dyn Fn(MutationEvent, &mut State, &mut MutationState)>;
