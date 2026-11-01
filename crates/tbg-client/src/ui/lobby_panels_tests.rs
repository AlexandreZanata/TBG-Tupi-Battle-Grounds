//! Contract: every panel of the lobby home is hidden while the Ranking view is
//! open (`ranking.css`). A panel missing from that rule stays on top of the
//! Ranking page and covers its close button (the P47 Live battles panel
//! shipped that way), so the player cannot close the page.

const PAGE: &str = include_str!("../../index.html");
const RANKING_CSS: &str = include_str!("../../assets/ranking/ranking.css");

/// First class of each direct child panel of `#tbg-lobby` (`main`, `section`
/// or `aside` with a class, indented two spaces in the page).
fn home_panels() -> Vec<&'static str> {
    let start = PAGE.find("id=\"tbg-lobby\"").expect("the lobby element");
    let end = PAGE.find("id=\"tbg-battle\"").expect("the battle element");
    PAGE[start..end]
        .lines()
        .filter(|line| line.starts_with("  <") && !line.starts_with("   "))
        .filter_map(|line| {
            let tag = ["<main class=\"", "<section class=\"", "<aside class=\""]
                .into_iter()
                .find(|tag| line[2..].starts_with(tag))?;
            let classes = &line[2 + tag.len()..];
            classes.split(['"', ' ']).next()
        })
        .collect()
}

#[test]
fn every_home_panel_is_hidden_by_the_ranking_view() {
    let panels = home_panels();
    assert!(panels.len() >= 5, "the scan found {panels:?}");
    let rule = RANKING_CSS
        .lines()
        .find(|line| line.contains("data-lobby-view=\"ranking\"") && line.contains("display:none"))
        .expect("the rule that hides the home panels in the Ranking view");
    for panel in panels {
        assert!(
            rule.contains(&format!(">.{panel}")),
            "the home panel .{panel} is not hidden when the Ranking view opens (assets/ranking/ranking.css)"
        );
    }
}
