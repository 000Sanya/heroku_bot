use std::fmt::Display;

pub trait ResultExtension: Sized {
    type Ok;
    type Error: Display;

    #[track_caller]
    fn on_error<F: FnOnce(&Self::Error)>(self, f: F) -> Self;

    #[track_caller]
    fn log_on_error(self, message: &str) -> Self {
        self.on_error(|e| log::error!("{}: {}", message, e))
    }

    #[track_caller]
    fn log_and_expect(self, message: &str) -> Self::Ok;
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

    #[track_caller]
    fn log_and_expect(self, message: &str) -> T {
        match self.log_on_error(message) {
            Ok(t) => t,
            Err(_) => panic!(),
        }
    }
}
