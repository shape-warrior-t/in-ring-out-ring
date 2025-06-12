use std::{error::Error, time::Duration};

use kira::{
    AudioManager, DefaultBackend, Easing, StartTime, Tween,
    clock::{ClockHandle, ClockSpeed},
    sound::{
        FromFileError,
        static_sound::StaticSoundData,
        streaming::{StreamingSoundData, StreamingSoundHandle},
    },
};
use rand::Rng;

mod files {
    pub const HIGH_DRUM: &str = "high_drum.mp3";
    pub const LOW_DRUM: &str = "low_drum.mp3";
    pub const DEATH: &str = "death.mp3";
    pub const MUSIC: &str = "beta_level.wav";
}

const INSTANT_TWEEN: Tween = Tween {
    start_time: StartTime::Immediate,
    duration: Duration::ZERO,
    easing: Easing::Linear,
};

type AudioResult<T> = Result<T, Box<dyn Error>>;

pub struct Sounds {
    pub high_drum: StaticSoundData,
    pub low_drum: StaticSoundData,
    pub death: StaticSoundData,
}

impl Sounds {
    pub fn new() -> AudioResult<Self> {
        Ok(Self {
            high_drum: StaticSoundData::from_file(files::HIGH_DRUM)?,
            low_drum: StaticSoundData::from_file(files::LOW_DRUM)?,
            death: StaticSoundData::from_file(files::DEATH)?,
        })
    }
}

#[derive(Debug)]
struct MusicProgress {
    curr: i32,
    record: i32,
    limit: i32,
}

#[derive(Clone, Copy, Debug)]
pub enum Tick {
    Countdown(u64),
    Beat(u64),
}

pub struct Speaker {
    manager: AudioManager,
    beats_per_second: f64,
    music: StreamingSoundHandle<FromFileError>,
    clock: ClockHandle,
    num_ticks_processed: u64,
    progress: MusicProgress,
}

impl Speaker {
    pub fn new(bpm: f64) -> AudioResult<Self> {
        let mut manager = AudioManager::<DefaultBackend>::new(Default::default())?;
        let clock = manager.add_clock(ClockSpeed::TicksPerMinute(bpm))?;
        let music = StreamingSoundData::from_file(files::MUSIC)?;
        let seconds = music.duration().as_secs_f64();
        let beats_per_second = bpm / 60.0;
        let progress = MusicProgress {
            curr: 0,
            record: 0,
            limit: (seconds * beats_per_second / 16.0).round() as i32,
        };
        let mut music = manager.play(music)?;
        music.stop(INSTANT_TWEEN);
        Ok(Speaker {
            manager,
            beats_per_second,
            music,
            clock,
            num_ticks_processed: 0,
            progress,
        })
    }

    pub fn tick(&self, countdown_length: u64) -> Tick {
        let t = self.clock.time().ticks;
        if t < countdown_length {
            Tick::Countdown(t)
        } else {
            Tick::Beat(t - countdown_length)
        }
    }

    pub fn beat_fraction(&self) -> f64 {
        self.clock.time().fraction
    }

    pub fn update_music_progress(&mut self, beat: u64) {
        if beat != 0 && beat % 16 == 0 {
            let progress = &mut self.progress;
            progress.curr = (progress.curr + 1) % progress.limit;
            progress.record = progress.record.max(progress.curr);
        }
    }

    pub fn process_tick(&mut self, countdown_length: u64) -> Option<Tick> {
        if self.num_ticks_processed > self.clock.time().ticks {
            return None;
        }
        self.num_ticks_processed += 1;
        Some(self.tick(countdown_length))
    }

    pub fn restart_clock(&mut self) {
        self.num_ticks_processed = 0;
        self.clock.stop();
        self.clock.start();
    }

    pub fn play_sound(&mut self, sound: &StaticSoundData) -> AudioResult<()> {
        self.manager.play(sound.clone())?;
        Ok(())
    }

    pub fn play_music(&mut self, rng: &mut impl Rng) -> AudioResult<()> {
        let progress = &mut self.progress;
        progress.curr = rng.random_range(0..=progress.record);
        let beats = (progress.curr * 16) as f64;
        let seconds_per_beat = 1.0 / self.beats_per_second;
        let music = StreamingSoundData::from_file(files::MUSIC)?
            .loop_region(..)
            .start_position(beats * seconds_per_beat);
        self.music = self.manager.play(music)?;
        Ok(())
    }

    pub fn stop_music(&mut self) {
        self.music.stop(INSTANT_TWEEN);
    }
}
