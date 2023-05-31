pub use user_error_derive::UserError;

pub trait UserError {
    fn user_error(&self) -> String;
}

pub struct StringUserError(pub String);

impl UserError for StringUserError {
    fn user_error(&self) -> String {
        self.0.clone()
    }
}
