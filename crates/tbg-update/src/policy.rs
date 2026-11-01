//! Last verified monotonic security floor; revoked fallback builds must never execute.

use crate::{fsx, helper, layout::Layout};
use serde::{Deserialize, Serialize};
use std::io;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Policy {
    schema: u32,
    floor: u64,
}

pub fn floor(layout: &Layout) -> io::Result<u64> {
    let path = layout.state().join("security-floor.json");
    if !path.try_exists()? {
        return Ok(0);
    }
    let policy: Policy = fsx::read_json(&path, 4096)?;
    if policy.schema != 1 {
        return Err(io::Error::other("unsupported security policy"));
    }
    Ok(policy.floor)
}

/// Call only after TUF verification of the descriptor (not unsigned version discovery).
pub fn remember(layout: &Layout, value: u64) -> io::Result<u64> {
    let old = floor(layout)?;
    let floor = old.max(value);
    if floor != old {
        fsx::private_dir(&layout.state())?;
        fsx::write_json(
            &layout.state().join("security-floor.json"),
            &Policy { schema: 1, floor },
        )?;
    }
    Ok(floor)
}

pub fn permits_fallback(layout: &Layout, build: &str) -> bool {
    let Ok(floor) = floor(layout) else {
        return false;
    };
    helper::installed(layout, build).is_some_and(|d| {
        d.release_sequence >= floor
            && d.local_state_reader
                .contains(&crate::descriptor::LOCAL_STATE)
    })
}
