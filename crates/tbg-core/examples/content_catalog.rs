//! Export canonical display identifiers and form names for the portrait/catalog asset pipeline.

fn main() {
    for def in tbg_core::ruleset::CANON.units {
        print!(
            "{:?}|{}|{:?}|{:?}",
            def.kind, def.name, def.faction, def.archetype
        );
        for form in &def.forms {
            print!("|{}", form.name);
        }
        println!();
    }
}
