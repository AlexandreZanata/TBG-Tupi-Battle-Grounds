//! `tbg-assets`: asset definition schemas and validation contracts.
//!
//! Phase 00 carries the schema version only. Unit/faction/balance schemas
//! and GLB acceptance gates arrive with the asset factory (Phase 02).

/// Current version of the asset contract schemas.
pub const SCHEMA_VERSION: u32 = 0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schemas_start_unversioned_at_zero() {
        assert_eq!(SCHEMA_VERSION, 0);
    }
}
