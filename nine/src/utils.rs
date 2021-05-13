pub(crate) trait Errorful {
    fn if_err(self, f: impl FnOnce() -> ()) -> Self;
}

impl<T, E> Errorful for Result<T, E> {
    fn if_err(self, f: impl FnOnce() -> ()) -> Self {
        if self.is_err() {
            f();
        }
        self
    }
}