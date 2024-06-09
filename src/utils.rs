use std::fmt::Display;

pub trait ResultExtension: Sized {
    type Ok;
    type Error: Display;

    #[track_caller]
    fn on_error<F: FnOnce(&Self::Error)>(self, f: F) -> Self;
}

impl<T, E: Display> ResultExtension for Result<T, E> {
    type Ok = T;
    type Error = E;

    #[track_caller]
    fn on_error<F: FnOnce(&Self::Error)>(self, f: F) -> Self {
        if let Err(e) = &self {
            f(e)
        }
        self
    }
}
