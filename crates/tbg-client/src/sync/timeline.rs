//! Render timeline for snapshot interpolation (P30-C). Views are placed on
//! the server's own clock (`MatchView::clock`, 20 steps per second), and the
//! battlefield is drawn a short, adaptive delay behind the newest view, so
//! uneven arrival never freezes units between views or makes them jump when
//! a view comes early. Purely visual: positions and events are the server's.

use std::collections::VecDeque;

use tbg_core::content::TICK_HZ;
use tbg_core::{Event, MatchView};

/// Server seconds per simulation step.
const STEP: f64 = 1.0 / TICK_HZ as f64;
/// Views are 10 Hz, so interpolation needs at least one view interval.
const MIN_DELAY: f64 = 0.10;
/// A worse network degrades into short stalls instead of more lag.
const MAX_DELAY: f64 = 0.35;
/// Jitter samples (~3 s of views) for the adaptive delay.
const WINDOW: usize = 32;
/// The transit floor follows a slower path by this share per view. The VPS
/// trace has the server clock falling 2.2 s behind wall time in a burst of
/// late ticks (VM CPU steal): a sliding minimum lags that by its whole
/// window and starves; this follows in about a second while one late view
/// lifts it by only a tenth of its delay.
const RISE: f64 = 0.1;
/// Playback may run this much faster or slower to follow its target.
const RATE: f64 = 0.10;
/// Beyond this distance from its target the render clock jumps.
const SNAP: f64 = 0.5;
/// A view this far behind the newest one belongs to another timeline.
const RESET: f64 = 1.0;

/// What one frame draws: `from` is the latest view at or before the render
/// time, `to` the next one (the same view when nothing newer exists).
pub struct Frame<'a> {
    pub from: &'a MatchView,
    pub to: &'a MatchView,
    pub alpha: f32,
}

/// Counters for the diagnostics overlay (`?measure`).
#[derive(Clone, Copy, Debug, Default)]
pub struct TimelineStats {
    /// Frames drawn at the newest view because nothing newer had arrived.
    pub starved: u32,
    /// Render-clock jumps (reconnects, long stalls).
    pub snaps: u32,
    /// Views older than one already received (their events still play).
    pub stale: u32,
    pub delay_ms: f32,
    pub depth: usize,
}

#[derive(Default)]
pub struct Timeline {
    views: VecDeque<(f64, MatchView)>,
    /// Events wait for the render time of the view that carried them.
    events: VecDeque<(f64, Vec<Event>)>,
    /// Lower envelope of `arrival - server time`: the transit of an
    /// undelayed view.
    base: Option<f64>,
    /// How much later than `base` recent views arrived (jitter).
    excess: VecDeque<f64>,
    delay: f64,
    render: Option<f64>,
    pub stats: TimelineStats,
}

impl Timeline {
    /// Add a view received at local time `now` (seconds).
    pub fn push(&mut self, now: f64, view: MatchView) {
        let t = f64::from(view.clock) * STEP;
        let newest = self.views.back().map_or(f64::NEG_INFINITY, |v| v.0);
        if t + RESET < newest {
            // A new match, or a server restart: start over.
            *self = Timeline {
                stats: self.stats,
                ..Timeline::default()
            };
        }
        self.sample(now - t);
        if t <= newest && !self.views.is_empty() {
            self.stats.stale += 1;
            self.events.push_back((f64::NEG_INFINITY, view.events));
            return;
        }
        let events = view.events.clone();
        self.events.push_back((t, events));
        self.views.push_back((t, view));
        self.stats.depth = self.views.len();
    }

    fn sample(&mut self, transit: f64) {
        let base = match self.base {
            Some(b) if transit > b => b + (transit - b) * RISE,
            _ => transit,
        };
        self.base = Some(base);
        self.excess.push_back(transit - base);
        if self.excess.len() > WINDOW {
            self.excess.pop_front();
        }
        let mut excess: Vec<f64> = self.excess.iter().copied().collect();
        excess.sort_by(f64::total_cmp);
        let p95 = excess[(excess.len() * 95 / 100).min(excess.len() - 1)];
        self.delay = (MIN_DELAY + p95 + 0.01).clamp(MIN_DELAY, MAX_DELAY);
        self.stats.delay_ms = (self.delay * 1000.0) as f32;
    }

    /// Advance the render clock to local time `now` (`dt` since the last
    /// frame) and return what to draw. Never moves backwards.
    pub fn frame(&mut self, now: f64, dt: f64) -> Option<Frame<'_>> {
        let (&(oldest, _), &(newest, _)) = (self.views.front()?, self.views.back()?);
        let target = now - self.base.unwrap_or(0.0) - self.delay;
        let next = match self.render {
            None => target,
            Some(r) => {
                let diff = target - (r + dt);
                if diff.abs() > SNAP {
                    target
                } else {
                    r + dt + diff.clamp(-RATE * dt, RATE * dt)
                }
            }
        };
        let floor = self.render.unwrap_or(f64::NEG_INFINITY).max(oldest);
        let mut render = next.max(floor);
        if render > newest {
            self.stats.starved += 1;
            render = newest;
        }
        if self.render.is_some_and(|r| (render - r - dt).abs() > SNAP) {
            self.stats.snaps += 1;
        }
        self.render = Some(render);
        // Keep the view at or before the render time; drop older ones.
        while self.views.len() > 1 && self.views[1].0 <= render {
            self.views.pop_front();
        }
        self.stats.depth = self.views.len();
        let (from_t, from) = (&self.views[0].0, &self.views[0].1);
        let (to_t, to) = self.views.get(1).map_or((from_t, from), |v| (&v.0, &v.1));
        let span = to_t - from_t;
        let alpha = if span > 0.0 {
            ((render - from_t) / span).clamp(0.0, 1.0) as f32
        } else {
            1.0
        };
        Some(Frame { from, to, alpha })
    }

    /// Events of every view the render clock has reached, in arrival order.
    pub fn take_events(&mut self) -> Vec<Event> {
        let render = self.render.unwrap_or(f64::NEG_INFINITY);
        let mut out = Vec::new();
        while self.events.front().is_some_and(|e| e.0 <= render) {
            if let Some((_, ev)) = self.events.pop_front() {
                out.extend(ev);
            }
        }
        out
    }
}

#[cfg(test)]
mod tests;
