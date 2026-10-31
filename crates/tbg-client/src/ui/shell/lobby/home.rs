//! Home-screen placement matches the browser and leaves the middle arena unobscured.

use super::{cards, header, modes, profile, Account, Data, Lang, Shell};
use bevy::prelude::*;

pub fn spawn(
    root: &mut ChildSpawnerCommands,
    shell: &Shell,
    data: &Data,
    account: &Account,
    assets: &AssetServer,
    lang: Lang,
    muted: bool,
) {
    header::spawn(root, data, account, assets, lang, muted);
    modes::spawn(root, account, assets, lang);
    profile::spawn(root, data, account, assets, lang);
    cards::features(root, assets, lang);
    cards::live(root, data, lang);
    cards::news(root, shell.news, assets, lang);
}
