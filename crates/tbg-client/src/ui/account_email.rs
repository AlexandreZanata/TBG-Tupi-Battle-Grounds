//! Protected email-change wizard state, independent from public account/profile DTOs.
#[derive(Default)]
pub struct Flow {
    pub key: String,
    pub email: String,
    pub old_code: String,
    pub new_code: String,
    pub hold: bool,
}
