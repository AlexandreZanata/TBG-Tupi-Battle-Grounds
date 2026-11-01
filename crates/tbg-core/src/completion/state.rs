//! Bounded replay state and public reconnect facts for champion completion.

use crate::{Seat, Tick, Unit, UnitId, UnitKind};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Care {
    pub amount: i32,
    pub until: Tick,
    pub seat: Option<Seat>,
    pub producer: Option<UnitKind>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Regrowth {
    pub amount: i32,
    pub next: Tick,
    pub left: u8,
    pub seat: Option<Seat>,
    pub producer: Option<UnitKind>,
    pub gap: u16,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Spark {
    pub amount: i32,
    pub until: Tick,
    pub source: UnitId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Channel {
    pub target: UnitId,
    pub amount: i32,
    pub next: Tick,
    pub left: u8,
    pub gap: u16,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fx {
    pub echo: Care,
    pub regrowth: Regrowth,
    pub spark: Spark,
    pub interrupt_until: Tick,
    pub walk_ticks: u16,
    pub charged: bool,
    pub prepared_heal: i32,
    pub channel: Option<Channel>,
    pub beam_center: i32,
    pub combat_death: bool,
    pub burst_suppressed: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct View {
    pub echo_ticks: u16,
    pub regrowth_pulses: u8,
    pub spark_ticks: u16,
    pub interrupt_ticks: u16,
    pub charged: bool,
    pub channel_target: Option<UnitId>,
    pub channel_pulses: u8,
    pub beam_center: Option<i32>,
}

pub fn view(u: &Unit, now: Tick) -> Option<View> {
    let left = |until: Tick| until.saturating_sub(now).min(u32::from(u16::MAX)) as u16;
    let fx = &u.fx.completion;
    let result = View {
        echo_ticks: if fx.echo.amount > 0 {
            left(fx.echo.until)
        } else {
            0
        },
        regrowth_pulses: fx.regrowth.left,
        spark_ticks: if fx.spark.amount > 0 {
            left(fx.spark.until)
        } else {
            0
        },
        interrupt_ticks: left(fx.interrupt_until),
        charged: fx.charged,
        channel_target: fx.channel.map(|c| c.target),
        channel_pulses: fx.channel.map_or(0, |c| c.left),
        beam_center: (u.kind == UnitKind::Prismancer && u.fx.special.is_some())
            .then_some(fx.beam_center),
    };
    (result != View::default()).then_some(result)
}

/// Fusion preserves victim-side families without carrying a parent's action sequence.
pub(crate) fn inherit(a: Fx, b: Fx, now: Tick) -> Fx {
    let live_echo = |value: Care| {
        if value.until > now {
            value
        } else {
            Care::default()
        }
    };
    let (ae, be) = (live_echo(a.echo), live_echo(b.echo));
    let mut echo = if be.amount > ae.amount { be } else { ae };
    echo.until = ae.until.max(be.until);
    let live_spark = |value: Spark| {
        if value.until > now {
            value
        } else {
            Spark::default()
        }
    };
    let (a_spark, b_spark) = (live_spark(a.spark), live_spark(b.spark));
    let mut spark = if b_spark.amount > a_spark.amount {
        b_spark
    } else {
        a_spark
    };
    spark.until = a_spark.until.max(b_spark.until);
    let mut regrowth = match (a.regrowth.left, b.regrowth.left) {
        (0, _) => b.regrowth,
        (_, 0) => a.regrowth,
        _ => {
            let mut winner = if b.regrowth.amount > a.regrowth.amount {
                b.regrowth
            } else {
                a.regrowth
            };
            winner.left = a.regrowth.left.max(b.regrowth.left);
            winner.next = a.regrowth.next.min(b.regrowth.next);
            winner
        }
    };
    if regrowth.left == 0 {
        regrowth = Regrowth::default();
    }
    Fx {
        echo,
        spark,
        regrowth,
        interrupt_until: a.interrupt_until.max(b.interrupt_until),
        ..Fx::default()
    }
}
