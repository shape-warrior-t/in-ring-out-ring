use std::collections::VecDeque;

use rand::Rng;

use crate::level::{Attack, AttackPatterns, Origin, TransformBlueprint};
use crate::playfield::{Coord, Pattern, Transform};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flash {
    Warn,
    Strike,
}

#[derive(Debug)]
pub enum Command<const N: usize> {
    NewAttack(Attack<N>, Transform<N>),
    FlashPattern(Pattern<N>, Flash),
}

impl<const N: usize> Command<N> {
    fn warn(pattern: Pattern<N>) -> Self {
        Command::FlashPattern(pattern, Flash::Warn)
    }

    fn strike(pattern: Pattern<N>) -> Self {
        Command::FlashPattern(pattern, Flash::Strike)
    }
}

impl TransformBlueprint {
    pub fn construct<const N: usize>(
        &self,
        rng: &mut impl Rng,
        player: (Coord<N>, Coord<N>),
    ) -> Transform<N> {
        Transform {
            origin: match self.origin {
                Origin::Random => rng.random(),
                Origin::Targeted => player,
            },
            transpose: self.transpose && rng.random_bool(0.5),
            mirror: self.mirror && rng.random_bool(0.5),
        }
    }
}

impl<const N: usize> Attack<N> {
    pub fn beat_length(&self) -> u64 {
        match self.patterns {
            AttackPatterns::Four(_) => 4,
            AttackPatterns::FourPlusFour(_) => 8,
            AttackPatterns::Eight(_) => 8,
            AttackPatterns::EightPlusEight(_) => 16,
        }
    }

    pub fn weight(&self) -> f32 {
        match self.patterns {
            AttackPatterns::Four(_) => 0.25,
            AttackPatterns::FourPlusFour(_) => 0.5,
            AttackPatterns::Eight(_) => 0.5,
            AttackPatterns::EightPlusEight(_) => 1.0,
        }
    }

    pub fn enqueue(self, commands: &mut VecDeque<Command<N>>, transform: Transform<N>) {
        commands.push_back(Command::NewAttack(self.clone(), transform));
        match self.patterns {
            AttackPatterns::Four([pattern]) => {
                let pattern = pattern.construct(transform);
                for _ in 0..3 {
                    commands.push_back(Command::warn(pattern.clone()));
                }
                commands.push_back(Command::strike(pattern));
            }
            AttackPatterns::FourPlusFour(patterns) => {
                let patterns = patterns.map(|p| p.construct(transform));
                commands.extend(patterns.clone().map(Command::warn));
                commands.push_back(Command::warn(Pattern::empty()));
                commands.extend(patterns.map(Command::strike));
                commands.push_back(Command::strike(Pattern::empty()));
            }
            AttackPatterns::Eight([pattern]) => {
                let pattern = pattern.construct(transform);
                for _ in 0..7 {
                    commands.push_back(Command::warn(pattern.clone()));
                }
                commands.push_back(Command::strike(pattern));
            }
            AttackPatterns::EightPlusEight(patterns) => {
                let patterns = patterns.map(|p| p.construct(transform));
                commands.extend(patterns.clone().map(Command::warn));
                commands.push_back(Command::warn(Pattern::empty()));
                commands.push_back(Command::warn(Pattern::empty()));
                commands.extend(patterns.map(Command::strike));
                commands.push_back(Command::strike(Pattern::empty()));
                commands.push_back(Command::strike(Pattern::empty()));
            }
        }
    }
}
