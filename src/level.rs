use std::{error::Error, str::FromStr};

use macroquad::color::Color;
use serde::Deserialize;
use serde_with::{BoolFromInt, TryFromInto, serde_as};

use crate::playfield::PatternBlueprint;

#[derive(Debug, Deserialize)]
struct SerializationColors(f32, f32, f32);

impl TryFrom<SerializationColors> for Color {
    type Error = String;

    fn try_from(value: SerializationColors) -> Result<Self, Self::Error> {
        let SerializationColors(r, g, b) = value;
        if !(0.0..=1.0).contains(&r) {
            return Err(format!("color r must be between 0 and 1, got {r}"));
        }
        if !(0.0..=1.0).contains(&g) {
            return Err(format!("color g must be between 0 and 1, got {g}"));
        }
        if !(0.0..=1.0).contains(&b) {
            return Err(format!("color b must be between 0 and 1, got {b}"));
        }
        Ok(Color::new(r, g, b, 1.0))
    }
}

#[serde_as]
#[derive(Debug, Deserialize)]
pub struct LevelColors<const N: usize> {
    #[serde_as(as = "TryFromInto<SerializationColors>")]
    pub background: Color,
    #[serde_as(as = "TryFromInto<SerializationColors>")]
    pub out_ring: Color,
    #[serde_as(as = "TryFromInto<SerializationColors>")]
    pub player: Color,
    #[serde_as(as = "TryFromInto<SerializationColors>")]
    pub flash: Color,
    #[serde_as(as = "[TryFromInto<SerializationColors>; N]")]
    pub main: [Color; N],
}

#[derive(Clone, Debug, Deserialize)]
#[serde(from = "SerializationAttackPatterns<N>")]
pub enum AttackPatterns<const N: usize> {
    Four([PatternBlueprint<N>; 1]),
    FourPlusFour([PatternBlueprint<N>; 3]),
    Eight([PatternBlueprint<N>; 1]),
    EightPlusEight([PatternBlueprint<N>; 6]),
}

#[derive(Debug, Deserialize)]
#[serde(tag = "beats", content = "patterns")]
enum SerializationAttackPatterns<const N: usize> {
    #[serde(rename = "4")]
    Four(SerializationPatterns<N, 1>),
    #[serde(rename = "4+4")]
    FourPlusFour(SerializationPatterns<N, 3>),
    #[serde(rename = "8")]
    Eight(SerializationPatterns<N, 1>),
    #[serde(rename = "8+8")]
    EightPlusEight(SerializationPatterns<N, 6>),
}

#[serde_as]
#[derive(Debug, Deserialize)]
struct SerializationPatterns<const N: usize, const P: usize>(
    #[serde_as(as = "[[[BoolFromInt; N]; P]; N]")] [[[bool; N]; P]; N],
);

impl<const N: usize, const P: usize> From<SerializationPatterns<N, P>>
    for [PatternBlueprint<N>; P]
{
    #[allow(clippy::needless_range_loop)]
    fn from(value: SerializationPatterns<N, P>) -> Self {
        let SerializationPatterns(value) = value;
        let mut result = [[[false; N]; N]; P];
        for o in 0..N {
            for p in 0..P {
                for i in 0..N {
                    result[p][o][i] = value[o][p][i];
                }
            }
        }
        result.map(PatternBlueprint::new)
    }
}

impl<const N: usize> From<SerializationAttackPatterns<N>> for AttackPatterns<N> {
    fn from(value: SerializationAttackPatterns<N>) -> Self {
        use AttackPatterns as AP;
        use SerializationAttackPatterns as SAP;
        match value {
            SAP::Four(patterns) => AP::Four(patterns.into()),
            SAP::FourPlusFour(patterns) => AP::FourPlusFour(patterns.into()),
            SAP::Eight(patterns) => AP::Eight(patterns.into()),
            SAP::EightPlusEight(patterns) => AP::EightPlusEight(patterns.into()),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Origin {
    #[default]
    Random,
    Targeted,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct TransformBlueprint {
    pub origin: Origin,
    pub transpose: bool,
    pub mirror: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Attack<const N: usize> {
    #[serde(flatten)]
    pub patterns: AttackPatterns<N>,
    #[serde(flatten)]
    pub transform: TransformBlueprint,
}

#[derive(Debug, Deserialize)]
pub struct Level<const N: usize> {
    pub bpm: f64,
    pub colors: LevelColors<N>,
    pub attacks: Vec<Attack<N>>,
}

impl<const N: usize> FromStr for Level<N> {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let level: Self = serde_json::from_str(s)?;
        if level.bpm <= 0.0 {
            return Err(format!("bpm must be positive, got {}", level.bpm).into());
        }
        Ok(level)
    }
}
