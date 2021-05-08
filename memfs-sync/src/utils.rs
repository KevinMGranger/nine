//! Helper utilities.
use std::borrow::{Cow, ToOwned};

/// Be given a Cow for the "current" version of an object,
/// and visit it. If the visitor makes any changes and doesn't
/// error out, the new owned version will be returned.
/// This is only necessary because a new scope is needed to move the cow.
/// This will be deprecated once NLL lands.
pub fn atomic_maybe_change<T, F, E>(
    current: &T,
    examiner: F,
) -> Result<Option<<T as ToOwned>::Owned>, E>
where
    T: ToOwned,
    F: FnOnce(&mut Cow<'_, T>) -> Result<(), E>,
{
    let mut cow = Cow::Borrowed(current);

    (examiner)(&mut cow).map(|_| {
        if let Cow::Owned(x) = cow {
            Some(x)
        } else {
            None
        }
    })
}
