use crate::store;

use super::{errors::UserNotFoundError, state::User};

pub fn select_user<'a>(state: &'a store::State) -> Result<&'a User, UserNotFoundError> {
    state.user.user.as_ref().ok_or(UserNotFoundError)
}
