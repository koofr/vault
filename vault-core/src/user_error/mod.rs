use crate::intl;

pub trait UserError {
    fn user_error(&self, intl_service: &intl::IntlService) -> String;
}

pub struct FnUserError<F: Fn(&intl::IntlService) -> String>(pub F);

impl<F: Fn(&intl::IntlService) -> String> UserError for FnUserError<F> {
    fn user_error(&self, intl_service: &intl::IntlService) -> String {
        self.0(intl_service)
    }
}
