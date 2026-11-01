//! Exact, runtime-only lifetime participation and bounded public result cosmetics (BR-VC02).

use crate::{Seat, UnitKind, ALL_KINDS};
use serde::{
    de::{self, SeqAccess, Visitor},
    Deserialize, Deserializer, Serialize,
};

pub const CONTENT_VERSION: u8 = 1;
pub const MAX_BUCKETS: usize = ALL_KINDS.len() * 5 * 2;
pub const MAX_CAST_BYTES: usize = MAX_BUCKETS * 14 + 64;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CastEntry {
    pub kind: UnitKind,
    pub tier: u8,
    pub branch: u8,
    pub summoned: bool,
    pub count: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CelebrationCast {
    pub content_version: u8,
    pub winner: Seat,
    #[serde(deserialize_with = "entries")]
    pub entries: Vec<CastEntry>,
}

fn bucket(kind: UnitKind, tier: u8, branch: u8, summoned: bool) -> Option<usize> {
    if kind == UnitKind::Crate
        || tier > 2
        || branch > 1
        || (tier == 0 && branch != 0)
        || (kind.is_summon() && !summoned)
    {
        return None;
    }
    let kind = ALL_KINDS.iter().position(|k| *k == kind)?;
    let form = if tier == 0 {
        0
    } else {
        1 + usize::from(branch) * 2 + usize::from(tier - 1)
    };
    Some((kind * 5 + form) * 2 + usize::from(summoned))
}

impl CelebrationCast {
    /// Validate before expanding even one cosmetic figure. Counts never drive allocation.
    pub fn total(&self, winner: Seat) -> Option<u64> {
        if self.content_version != CONTENT_VERSION
            || self.winner != winner
            || self.entries.len() > MAX_BUCKETS
        {
            return None;
        }
        let (mut previous, mut total) = (None, 0u64);
        for e in &self.entries {
            let key = bucket(e.kind, e.tier, e.branch, e.summoned)?;
            if e.count == 0 || previous.is_some_and(|p| p >= key) {
                return None;
            }
            total = total.checked_add(e.count)?;
            previous = Some(key);
        }
        Some(total)
    }

    /// Resolve one logical index without materializing repeated participants.
    pub fn at(&self, mut index: u64) -> Option<&CastEntry> {
        for entry in &self.entries {
            if index < entry.count {
                return Some(entry);
            }
            index = index.checked_sub(entry.count)?;
        }
        None
    }
}

fn entries<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<CastEntry>, D::Error> {
    struct Bounded;
    impl<'de> Visitor<'de> for Bounded {
        type Value = Vec<CastEntry>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("bounded cast buckets")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
            if seq.size_hint().is_some_and(|n| n > MAX_BUCKETS) {
                return Err(de::Error::custom("cast bucket limit"));
            }
            let mut rows = Vec::new();
            while let Some(row) = seq.next_element()? {
                if rows.len() == MAX_BUCKETS {
                    return Err(de::Error::custom("cast bucket limit"));
                }
                rows.push(row);
            }
            Ok(rows)
        }
    }
    d.deserialize_seq(Bounded)
}

/// An opaque, bounded field isolates malformed cosmetics from the authoritative Outcome.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CastPayload(#[serde(deserialize_with = "payload")] Vec<u8>);

impl CastPayload {
    pub fn new(cast: &CelebrationCast) -> Option<Self> {
        cast.total(cast.winner)?;
        let bytes = postcard::to_allocvec(cast).ok()?;
        (bytes.len() <= MAX_CAST_BYTES).then_some(Self(bytes))
    }
    pub fn decode(&self, winner: Seat) -> Option<CelebrationCast> {
        let (cast, tail): (CelebrationCast, _) = postcard::take_from_bytes(&self.0).ok()?;
        if !tail.is_empty() {
            return None;
        }
        cast.total(winner)?;
        Some(cast)
    }
}

fn payload<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
    struct Bounded;
    impl Visitor<'_> for Bounded {
        type Value = Vec<u8>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("bounded cosmetic bytes")
        }
        fn visit_bytes<E: de::Error>(self, bytes: &[u8]) -> Result<Self::Value, E> {
            // A well-framed but unusable cosmetic field must not drop MatchEnd.
            Ok(if bytes.len() <= MAX_CAST_BYTES {
                bytes.to_vec()
            } else {
                Vec::new()
            })
        }
    }
    d.deserialize_bytes(Bounded)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LifetimeCast {
    counts: [Vec<u64>; 2],
    exact: bool,
}

impl Default for LifetimeCast {
    fn default() -> Self {
        Self {
            counts: std::array::from_fn(|_| vec![0; MAX_BUCKETS]),
            exact: false,
        }
    }
}

impl LifetimeCast {
    pub(crate) fn new() -> Self {
        Self {
            exact: true,
            ..Self::default()
        }
    }
    pub(crate) fn born(
        &mut self,
        seat: Seat,
        kind: UnitKind,
        tier: u8,
        branch: u8,
        summoned: bool,
    ) {
        if kind == UnitKind::Crate {
            return;
        }
        let Some(slot) = bucket(kind, tier, branch, summoned) else {
            self.exact = false;
            return;
        };
        let count = &mut self.counts[seat.index()][slot];
        match count.checked_add(1) {
            Some(n) => *count = n,
            None => self.exact = false,
        }
    }
    pub(crate) fn evolved(
        &mut self,
        seat: Seat,
        kind: UnitKind,
        tier: u8,
        old_branch: u8,
        branch: u8,
    ) {
        let (Some(old), Some(new)) = (
            bucket(kind, tier, old_branch, false),
            bucket(kind, tier + 1, branch, false),
        ) else {
            self.exact = false;
            return;
        };
        let counts = &mut self.counts[seat.index()];
        match (counts[old].checked_sub(1), counts[new].checked_add(1)) {
            (Some(a), Some(b)) => {
                counts[old] = a;
                counts[new] = b;
            }
            _ => self.exact = false,
        }
    }
    fn snapshot(&self, winner: Seat) -> Option<CelebrationCast> {
        if !self.exact {
            return None;
        }
        let mut entries = Vec::new();
        for kind in ALL_KINDS {
            for (tier, branch) in [(0, 0), (1, 0), (2, 0), (1, 1), (2, 1)] {
                for summoned in [false, true] {
                    let Some(slot) = bucket(kind, tier, branch, summoned) else {
                        continue;
                    };
                    let count = self.counts[winner.index()][slot];
                    if count > 0 {
                        entries.push(CastEntry {
                            kind,
                            tier,
                            branch,
                            summoned,
                            count,
                        });
                    }
                }
            }
        }
        let cast = CelebrationCast {
            content_version: CONTENT_VERSION,
            winner,
            entries,
        };
        cast.total(winner)?;
        Some(cast)
    }
}

impl crate::MatchState {
    pub fn celebration_cast(&self) -> Option<CelebrationCast> {
        self.participation.snapshot(self.outcome?.winner?)
    }
}
