use std::{
    cell::RefCell,
    fmt::Debug,
    hash::Hash,
    rc::Rc,
    sync::{Arc, Mutex, RwLock},
};

use super::{
    event_emitter::EventEmitter, mutation_event_emitter::MutationEventEmitter, next_id::NextId,
    MutationNotify, Notify,
};

pub type OnCallback<MutationState> = Box<dyn Fn(&MutationState) + Send + Sync>;

pub type OnMutationCallback<State, Event, MutationState, MutationEvent> = Box<
    dyn Fn(
            &mut State,
            &Notify<Event>,
            &mut MutationState,
            &MutationNotify<MutationEvent, State, MutationState>,
        ) + Send
        + Sync,
>;

pub struct Store<State, Event, MutationState, MutationEvent> {
    state: Arc<RwLock<State>>,
    event_emitter: EventEmitter<Event, MutationState>,
    mutation_event_emitter: Arc<MutationEventEmitter<State, Event, MutationState, MutationEvent>>,
    next_id: Arc<Mutex<NextId>>,
}

impl<State, Event, MutationState, MutationEvent> Store<State, Event, MutationState, MutationEvent>
where
    State: Debug + Clone + Send + Sync + 'static,
    Event: Debug + Clone + PartialEq + Eq + Hash + Send + 'static,
    MutationState: Debug + Clone + Default + 'static,
    MutationEvent: Debug + Clone + PartialEq + Eq + Hash + Send + 'static,
{
    pub fn new(initial_state: State) -> Self {
        Self {
            state: Arc::new(RwLock::new(initial_state)),
            event_emitter: EventEmitter::new(),
            mutation_event_emitter: Arc::new(MutationEventEmitter::new()),
            next_id: Arc::new(Mutex::new(Default::default())),
        }
    }

    pub fn get_next_id(&self) -> u32 {
        self.next_id.lock().unwrap().next()
    }

    pub fn on(&self, id: u32, events: &[Event], callback: OnCallback<MutationState>) {
        self.event_emitter.on(id, events, callback)
    }

    pub fn remove_listener(&self, id: u32) {
        self.event_emitter.remove_listener(id)
    }

    pub fn mutation_on(
        &self,
        id: u32,
        events: &[MutationEvent],
        callback: OnMutationCallback<State, Event, MutationState, MutationEvent>,
    ) {
        let mutation_event_emitter = self.mutation_event_emitter.clone();

        self.mutation_event_emitter.on(
            id,
            events,
            Box::new(move |state, notify, mutation_state| {
                callback(
                    state,
                    notify.clone().as_ref(),
                    mutation_state,
                    &Self::get_mutation_notify(mutation_event_emitter.clone(), notify.clone()),
                );
            }),
        );
    }

    pub fn mutation_remove_listener(&self, id: u32) {
        self.mutation_event_emitter.remove_listener(id)
    }

    pub fn with_state<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&State) -> R,
    {
        let state = self.state.read().unwrap();

        f(&state)
    }

    pub fn mutate<F, R>(&self, f: F) -> R
    where
        F: FnOnce(
            &mut State,
            &Notify<Event>,
            &mut MutationState,
            &MutationNotify<MutationEvent, State, MutationState>,
        ) -> R,
    {
        let mut state = self.state.write().unwrap();

        let events: Rc<RefCell<Vec<Event>>> = Rc::new(RefCell::new(Vec::with_capacity(1)));

        let notify_events = events.clone();
        let notify: Rc<Notify<Event>> = Rc::new(Box::new(move |event: Event| {
            let events = notify_events.clone();
            let mut events = events.borrow_mut();

            if !events.contains(&event) {
                events.push(event);
            }
        }));

        let mut mutation_state = MutationState::default();

        let res = f(
            &mut state,
            notify.clone().as_ref(),
            &mut mutation_state,
            &Self::get_mutation_notify(self.mutation_event_emitter.clone(), notify.clone()),
        );

        drop(state);

        let events = events.clone();

        for event in events.borrow().iter() {
            self.event_emitter.emit(event, &mutation_state);
        }

        res
    }

    fn get_mutation_notify(
        mutation_event_emitter: Arc<
            MutationEventEmitter<State, Event, MutationState, MutationEvent>,
        >,
        notify: Rc<Notify<Event>>,
    ) -> MutationNotify<MutationEvent, State, MutationState> {
        Box::new(move |event, state, mutation_state| {
            mutation_event_emitter.emit(&event, state, notify.clone(), mutation_state);
        })
    }
}
