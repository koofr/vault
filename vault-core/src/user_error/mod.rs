pub use user_error_derive::UserError;

pub trait UserError {
    fn user_error(&self) -> String;
}
