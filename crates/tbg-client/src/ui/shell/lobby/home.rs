//! Home-screen placement matches the browser and leaves the middle arena unobscured.

#[cfg(not(target_os = "linux"))]
use super::super::header;
use super::{cards, modes, profile, Account, Data, Lang, Shell};
use bevy::prelude::*;

pub fn spawn(
    root: &mut ChildSpawnerCommands,
    shell: &Shell,
    data: &Data,
    account: &Account,
    assets: &AssetServer,
    lang: Lang,
) {
    #[cfg(not(target_os = "linux"))]
    header::spawn(root, data, account, assets, lang);
    modes::spawn(root, account, assets, lang);
    profile::spawn(root, data, account, assets, lang);
    cards::features(root, assets, lang);
    cards::live(root, data, lang);
    cards::news(root, shell.news, assets, lang);
}
