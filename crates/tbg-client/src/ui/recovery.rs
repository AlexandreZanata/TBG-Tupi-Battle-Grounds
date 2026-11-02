//! Lost-key wizard state; codes and credentials stay in memory and use the auth coordinator.
#[derive(Default)]
pub struct Flow {
    pub name: String,
    pub email: String,
    pub requested: bool,
}
