//! Android prototype deliberately has no durable credential provider.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Missing,
    Unavailable,
    Invalid,
}
pub fn read(_namespace: &str) -> Result<String, Error> {
    Err(Error::Missing)
}
pub fn read_within(namespace: &str, _limit: std::time::Duration) -> Result<String, Error> {
    read(namespace)
}
pub fn write(_namespace: &str, _value: &str) -> Result<(), Error> {
    Err(Error::Unavailable)
}
pub fn remove(_namespace: &str) -> Result<(), Error> {
    Ok(())
}
