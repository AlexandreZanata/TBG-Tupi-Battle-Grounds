//! P30-C motion harness: replays view arrival schedules (synthetic jitter
//! and a real trace from the VPS) at 60 FPS through the render timeline
//! and through the pre-P30-C arrival-time formula, and measures what a
//! player sees: stalled frames, jumps, backward steps and visual lag.

use tbg_core::{Event, MatchState, MatchView, Seat, UnitKind, UnitView};

use super::{Timeline, MAX_DELAY};

/// Milli-tiles per server step (1 tile/s, a typical walking speed).
const SPEED: f64 = 50.0;
const FPS: f64 = 60.0;
const TRACE: &str = include_str!("vps-trace.txt");

fn unit(id: u32, x: i32) -> UnitView {
    UnitView {
        id,
        seat: Seat::A,
        kind: UnitKind::Archer,
        tier: 0,
        branch: 0,
        x,
        hp: 100,
        max_hp: 100,
        status: 0,
        fx: 0,
        carrier: None,
        shield: 0,
        reduction: 0,
        special: None,
        effects: None,
    }
}

/// A view at `clock` with one unit at `x`.
fn view(clock: u32, x: i32) -> MatchView {
    let mut v = MatchState::new(1).view_for(Seat::A, &[]);
    v.clock = clock;
    v.units = vec![unit(1, x)];
    v
}

/// The unit walks at `SPEED` on the server clock.
fn walking(clock: u32) -> MatchView {
    view(clock, (f64::from(clock) * SPEED) as i32)
}

#[derive(Debug, Default)]
struct Motion {
    frames: u32,
    stalls: u32,
    jumps: u32,
    back: u32,
    lag_ms_mean: f64,
}

impl Motion {
    fn stall_pct(&self) -> f64 {
        100.0 * f64::from(self.stalls) / f64::from(self.frames.max(1))
    }
}

/// Score a displayed trajectory sampled every frame (milli-tiles); `lags`
/// is the visual delay per frame in ms. The first second is warm-up.
fn score(xs: &[f64], lags: &[f64]) -> Motion {
    let nominal = SPEED * 20.0 / FPS;
    let mut m = Motion::default();
    for (i, w) in xs.windows(2).enumerate().skip(FPS as usize) {
        let dx = w[1] - w[0];
        m.frames += 1;
        m.stalls += u32::from(dx.abs() < 0.5);
        m.jumps += u32::from(dx > 2.0 * nominal);
        m.back += u32::from(dx < -0.5);
        m.lag_ms_mean += lags[i + 1];
    }
    m.lag_ms_mean /= f64::from(m.frames.max(1));
    m
}

/// Deliver `(arrival s, view)` at 60 FPS frames; return per-frame x.
fn play_timeline(arrivals: &[(f64, MatchView)]) -> (Vec<f64>, Vec<f64>, Timeline) {
    let mut tl = Timeline::default();
    let (mut xs, mut lags) = (Vec::new(), Vec::new());
    let (start, end) = (arrivals[0].0, arrivals[arrivals.len() - 1].0);
    let mut next = 0;
    let mut now = start;
    while now <= end {
        while next < arrivals.len() && arrivals[next].0 <= now {
            tl.push(now, arrivals[next].1.clone());
            next += 1;
        }
        if let Some(f) = tl.frame(now, 1.0 / FPS) {
            let x0 = f64::from(f.from.units[0].x);
            let x1 = f.to.units.first().map_or(x0, |u| f64::from(u.x));
            let x = x0 + (x1 - x0) * f64::from(f.alpha);
            xs.push(x);
            lags.push(lag_ms(arrivals, now, x));
        }
        now += 1.0 / FPS;
    }
    (xs, lags, tl)
}

/// The pre-P30-C formula: interpolate the last two views by arrival time.
fn play_legacy(arrivals: &[(f64, MatchView)]) -> (Vec<f64>, Vec<f64>) {
    let mut prev: Option<(f64, i32)> = None;
    let mut last: Option<(f64, i32)> = None;
    let (mut xs, mut lags) = (Vec::new(), Vec::new());
    let (start, end) = (arrivals[0].0, arrivals[arrivals.len() - 1].0);
    let mut next = 0;
    let mut now = start;
    while now <= end {
        while next < arrivals.len() && arrivals[next].0 <= now {
            prev = last.take();
            last = Some((now, arrivals[next].1.units[0].x));
            next += 1;
        }
        if let Some((last_t, last_x)) = last {
            let (alpha, prev_x) = match prev {
                Some((pt, px)) => (
                    ((now - last_t) / (last_t - pt).max(0.05)).clamp(0.0, 1.0),
                    px,
                ),
                None => (1.0, last_x),
            };
            let x = f64::from(prev_x) + f64::from(last_x - prev_x) * alpha;
            xs.push(x);
            lags.push(lag_ms(arrivals, now, x));
        }
        now += 1.0 / FPS;
    }
    (xs, lags)
}

/// Visual delay: how far the shown position trails the newest known one.
fn lag_ms(arrivals: &[(f64, MatchView)], now: f64, x: f64) -> f64 {
    let newest = arrivals
        .iter()
        .take_while(|a| a.0 <= now)
        .last()
        .map_or(x, |a| f64::from(a.1.units[0].x));
    (newest - x) / SPEED * 50.0
}

/// Views every 100 ms of server time, delivered `jitter(i)` ms late.
fn schedule(n: u32, jitter: impl Fn(u32) -> f64) -> Vec<(f64, MatchView)> {
    let mut out: Vec<(f64, MatchView)> = (0..n)
        .map(|i| (f64::from(i) * 0.1 + jitter(i) / 1000.0, walking(i * 2)))
        .collect();
    // TCP delivers in order: a delayed view also holds back the next.
    for i in 1..out.len() {
        out[i].0 = out[i].0.max(out[i - 1].0);
    }
    out
}

/// Deterministic pseudo-random sequence in [0, 1).
fn noise(i: u32) -> f64 {
    let h = (u64::from(i) + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 11;
    h as f64 / (1u64 << 53) as f64
}

fn vps_trace() -> Vec<(f64, MatchView)> {
    TRACE
        .lines()
        .filter(|l| !l.starts_with('#'))
        .filter_map(|l| {
            let (a, t) = l.split_once(' ')?;
            Some((a.parse::<f64>().ok()? / 1000.0, walking(t.parse().ok()?)))
        })
        .collect()
}

#[test]
fn stable_100ms_views_move_without_stalls_or_jumps() {
    let (xs, lags, _) = play_timeline(&schedule(300, |_| 40.0));
    let m = score(&xs, &lags);
    assert_eq!((m.stalls, m.jumps, m.back), (0, 0, 0), "{m:?}");
    assert!(m.lag_ms_mean < 160.0, "{m:?}");
}

#[test]
fn moderate_jitter_stays_smooth_and_beats_the_old_formula() {
    // Arrival gaps between ~60 and ~140 ms.
    let arrivals = schedule(600, |i| 40.0 + 40.0 * noise(i));
    let (xs, lags, _) = play_timeline(&arrivals);
    let new = score(&xs, &lags);
    let (lx, ll) = play_legacy(&arrivals);
    let old = score(&lx, &ll);
    eprintln!("moderate legacy {old:?} / timeline {new:?}");
    assert!(
        new.stall_pct() < 1.0 && new.jumps == 0 && new.back == 0,
        "{new:?}"
    );
    assert!(
        old.stall_pct() > 5.0 * new.stall_pct().max(0.2),
        "{old:?} vs {new:?}"
    );
}

#[test]
fn high_jitter_degrades_predictably_and_never_runs_backwards() {
    // Every 20th view is held 250 ms; the others jitter by up to 60 ms.
    let arrivals = schedule(600, |i| if i % 20 == 0 { 250.0 } else { 60.0 * noise(i) });
    let (xs, lags, tl) = play_timeline(&arrivals);
    let m = score(&xs, &lags);
    assert_eq!(m.back, 0, "{m:?}");
    assert!(tl.stats.delay_ms <= (MAX_DELAY * 1000.0) as f32 + 1.0);
    assert!(tl.stats.depth < 12, "bounded buffer: {}", tl.stats.depth);
}

#[test]
fn the_real_vps_trace_is_smooth() {
    let arrivals = vps_trace();
    assert!(arrivals.len() > 2_000);
    let (xs, lags, tl) = play_timeline(&arrivals);
    let new = score(&xs, &lags);
    let (lx, ll) = play_legacy(&arrivals);
    let old = score(&lx, &ll);
    eprintln!("VPS legacy:   {old:?} stall {:.2}%", old.stall_pct());
    eprintln!(
        "VPS timeline: {new:?} stall {:.2}% {:?}",
        new.stall_pct(),
        tl.stats
    );
    // Over half of the remaining stalls fall in the ten seconds where the
    // server itself fell 2.2 s behind wall time: no newer state existed.
    assert!(new.stall_pct() < 1.5 && new.back == 0, "{new:?}");
    assert!(
        new.stall_pct() * 4.0 < old.stall_pct(),
        "{old:?} vs {new:?}"
    );
    assert!(new.jumps * 10 <= old.jumps.max(1), "{old:?} vs {new:?}");
}

#[test]
fn a_late_view_never_moves_time_backwards_and_keeps_its_events() {
    let mut tl = Timeline::default();
    tl.push(0.0, walking(0));
    tl.push(0.1, walking(2));
    tl.push(0.2, walking(6));
    let mut late = walking(4);
    late.events = vec![Event::RoundOpened { round: 7 }];
    tl.push(0.21, late);
    assert_eq!(tl.stats.stale, 1);
    let mut last = f64::NEG_INFINITY;
    let mut events = Vec::new();
    for i in 0..30 {
        let f = tl.frame(0.21 + f64::from(i) / FPS, 1.0 / FPS).unwrap();
        let x = f64::from(f.from.units[0].x)
            + f64::from(f.to.units[0].x - f.from.units[0].x) * f64::from(f.alpha);
        assert!(x >= last, "frame {i}: {x} < {last}");
        last = x;
        events.extend(tl.take_events());
    }
    assert!(events.contains(&Event::RoundOpened { round: 7 }));
}

#[test]
fn a_burst_of_views_in_one_frame_plays_in_order() {
    let mut arrivals = schedule(100, |_| 40.0);
    // Twenty views land in the same frame, after a two-second hole.
    for a in arrivals.iter_mut().take(60).skip(40) {
        a.0 = 4.54;
    }
    let (xs, lags, _) = play_timeline(&arrivals);
    let m = score(&xs, &lags);
    assert_eq!(m.back, 0, "{m:?}");
}

#[test]
fn spawn_and_death_follow_the_render_time() {
    let mut tl = Timeline::default();
    let mut views: Vec<MatchView> = (0..6).map(|i| walking(i * 2)).collect();
    views[2].units.push(unit(2, 500));
    views[2].events = vec![Event::Spawn {
        unit: 2,
        seat: Seat::A,
        kind: UnitKind::Archer,
    }];
    views[3].units.push(unit(2, 500));
    views[4].events = vec![Event::Death { unit: 2 }];
    for (i, v) in views.into_iter().enumerate() {
        tl.push(i as f64 * 0.1, v);
    }
    let (mut spawned, mut died) = (false, false);
    for i in 0..60 {
        let f = tl.frame(0.5 + f64::from(i) / FPS, 1.0 / FPS).unwrap();
        let has = f.from.units.iter().any(|u| u.id == 2);
        let clock = f.from.clock;
        let ev = tl.take_events();
        if ev.iter().any(|e| matches!(e, Event::Spawn { unit: 2, .. })) {
            spawned = true;
            assert!(has, "Spawn plays with the unit on screen");
        }
        if ev.contains(&Event::Death { unit: 2 }) {
            died = true;
            assert!(!has && clock == 8, "Death plays as the unit leaves");
        }
    }
    assert!(spawned && died);
}

#[test]
fn a_reconnect_snaps_once_and_a_new_match_starts_over() {
    let mut arrivals = schedule(50, |_| 40.0);
    // Ten seconds detached, then the catch-up view and live views again.
    for (i, a) in arrivals.iter_mut().enumerate().skip(25) {
        a.0 += 10.0;
        a.1 = walking(200 + i as u32 * 2);
    }
    let (xs, _, tl) = play_timeline(&arrivals);
    assert_eq!(tl.stats.snaps, 1);
    assert!(xs.windows(2).all(|w| w[1] >= w[0]));
    let mut tl = Timeline::default();
    tl.push(0.0, walking(5_000));
    tl.push(0.1, walking(0));
    assert_eq!(tl.stats.depth, 1, "a restarted clock clears the old match");
}

#[test]
fn an_augment_pause_freezes_and_resumes_without_a_jump() {
    // 3 s of combat, 2 s paused (clock runs, positions frozen), 3 s combat.
    let arrivals: Vec<(f64, MatchView)> = (0..80u32)
        .map(|i| {
            let clock = i * 2;
            let combat = clock.min(60) + clock.saturating_sub(100);
            (
                f64::from(i) * 0.1 + 0.04,
                view(clock, (f64::from(combat) * SPEED) as i32),
            )
        })
        .collect();
    let (xs, _, _) = play_timeline(&arrivals);
    let nominal = SPEED * 20.0 / FPS;
    assert!(xs
        .windows(2)
        .all(|w| w[1] - w[0] <= 2.0 * nominal && w[1] >= w[0]));
    let still = xs.windows(2).filter(|w| w[1] == w[0]).count();
    assert!(
        still as f64 >= 1.8 * FPS,
        "about two seconds frozen: {still}"
    );
}
