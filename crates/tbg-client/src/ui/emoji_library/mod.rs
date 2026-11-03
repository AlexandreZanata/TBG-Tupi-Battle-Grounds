//! Three-slot editing and upload selection state outside matches.
#[derive(Default)]
pub struct Library {
    pub slot: u8,
    pub label: String,
    pub file: Option<crate::platform::file_pick::Output>,
    pub selected: Option<(Vec<u8>, String)>,
}
