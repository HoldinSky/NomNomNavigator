use std::fmt::{Debug, Display, Formatter};

#[derive(Debug)]
pub struct PoolInitializationError(pub String);

impl Display for PoolInitializationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.pad(&self.0)
    }
}