mod audio;
mod command;
mod drawing;
mod level;
mod playfield;

use std::collections::VecDeque;

use audio::{Sounds, Speaker, Tick};
use command::{Command, Flash};
use drawing::Screen;
use level::{Attack, Level};
use macroquad::{
    color::WHITE,
    input::{KeyCode, is_key_pressed},
    time::get_frame_time,
    window::{clear_background, next_frame, request_new_screen_size, screen_height, screen_width},
};
use playfield::{Coord, Pattern, Transform};
use rand::{Rng, seq::IndexedRandom};

struct GameState<const N: usize> {
    play_state: PlayState<N>,
    player: (Coord<N>, Coord<N>),
    rotation: (f32, f32),
    rotation_speed: (f32, f32),
    pattern: Pattern<N>,
    flash: Flash,
    draw_flashes: bool,
    tick: Tick,
    high_score: u64,
}

enum PlayState<const N: usize> {
    Initial,
    Playing {
        commands: VecDeque<Command<N>>,
        curr_attack: Option<Attack<N>>,
        curr_transform: Transform<N>,
    },
    Death {
        commands: VecDeque<Command<N>>,
        attack: Option<Attack<N>>,
        original_transform: Transform<N>,
    },
    Transition(Box<PlayState<N>>),
}

fn transition_state<const N: usize>(play_state: PlayState<N>) -> PlayState<N> {
    PlayState::Transition(Box::new(play_state))
}

fn random_rotation(rng: &mut impl Rng) -> (f32, f32) {
    (rng.random_range(0.0..360.0), rng.random_range(0.0..360.0))
}

fn random_rotation_speed_slow(rng: &mut impl Rng) -> (f32, f32) {
    (
        rng.random_range(-60.0..=60.0),
        rng.random_range(-60.0..=60.0),
    )
}

fn random_rotation_speed_fast(rng: &mut impl Rng) -> (f32, f32) {
    (
        rng.random_range(-120.0..=120.0),
        rng.random_range(-120.0..=120.0),
    )
}

#[macroquad::main("In-Ring Out-Ring")]
async fn main() {
    const N: usize = 6;
    let level: Level<N> = std::fs::read_to_string("beta_level.json")
        .unwrap()
        .parse()
        .unwrap();
    request_new_screen_size(512.0, 512.0);
    let mut speaker = Speaker::new(level.bpm).unwrap();
    let sounds = Sounds::new().unwrap();
    let mut rng = rand::rng();
    let mut game_state = GameState {
        play_state: PlayState::Initial,
        player: rng.random(),
        rotation: random_rotation(&mut rng),
        rotation_speed: random_rotation_speed_slow(&mut rng),
        pattern: Pattern::empty(),
        flash: Flash::Warn,
        draw_flashes: false,
        tick: Tick::Beat(0),
        high_score: 0,
    };
    loop {
        game_state = update(game_state, &level, &mut speaker, &sounds, &mut rng);
        let screen = Screen::new(screen_width(), screen_height());
        draw(&screen, &game_state, &level);
        next_frame().await;
    }
}

fn new_game<const N: usize>(rng: &mut impl Rng, high_score: u64) -> GameState<N> {
    GameState {
        play_state: transition_state(PlayState::Playing {
            commands: VecDeque::new(),
            curr_attack: None,
            curr_transform: Default::default(),
        }),
        player: rng.random(),
        rotation: random_rotation(rng),
        rotation_speed: random_rotation_speed_fast(rng),
        pattern: Pattern::empty(),
        flash: Flash::Warn,
        draw_flashes: false,
        tick: Tick::Countdown(0),
        high_score,
    }
}

fn update<const N: usize>(
    gs: GameState<N>,
    level: &Level<N>,
    speaker: &mut Speaker,
    sounds: &Sounds,
    rng: &mut impl Rng,
) -> GameState<N> {
    enum GameResult {
        Playing,
        Death,
    }

    let gs = match gs.play_state {
        PlayState::Initial => {
            if is_key_pressed(KeyCode::Space) {
                new_game(rng, gs.high_score)
            } else {
                gs
            }
        }
        PlayState::Playing {
            mut commands,
            mut curr_attack,
            mut curr_transform,
        } => {
            let (mut player_i, mut player_o) = gs.player;
            let mut tick = gs.tick;
            let mut high_score = gs.high_score;
            let mut pattern = gs.pattern;
            let mut flash = gs.flash;
            let rotation = gs.rotation;
            let mut rotation_speed = gs.rotation_speed;
            #[allow(unused_variables)]
            let gs = ();
            if is_key_pressed(KeyCode::A) {
                player_i = player_i - Coord::ONE;
            }
            if is_key_pressed(KeyCode::D) {
                player_i = player_i + Coord::ONE;
            }
            if is_key_pressed(KeyCode::J) {
                player_o = player_o - Coord::ONE;
            }
            if is_key_pressed(KeyCode::L) {
                player_o = player_o + Coord::ONE;
            }
            let player = (player_i, player_o);
            let game_result = 'process: {
                if is_key_pressed(KeyCode::Backspace) {
                    break 'process GameResult::Death;
                }
                while let Some(next_tick) = speaker.process_tick(4) {
                    tick = next_tick;
                    match tick {
                        Tick::Countdown(0..3) => speaker.play_sound(&sounds.low_drum).unwrap(),
                        Tick::Countdown(3) => speaker.play_sound(&sounds.high_drum).unwrap(),
                        Tick::Countdown(_) => unreachable!(),
                        Tick::Beat(beat) => {
                            if beat == 0 {
                                speaker.play_music(rng).unwrap();
                            }
                            speaker.update_music_progress(beat);
                            high_score = high_score.max(beat);
                            if commands.is_empty() && !level.attacks.is_empty() {
                                let attack = level
                                    .attacks
                                    .choose_weighted(rng, |attack| {
                                        if beat % attack.beat_length() == 0 {
                                            attack.weight()
                                        } else {
                                            0.0
                                        }
                                    })
                                    .unwrap()
                                    .clone();
                                let transform = attack.transform.construct(rng, player);
                                attack.enqueue(&mut commands, transform);
                            }
                            loop {
                                let Some(command) = commands.pop_front() else {
                                    break;
                                };
                                match command {
                                    Command::NewAttack(attack, transform) => {
                                        curr_attack = Some(attack);
                                        curr_transform = transform;
                                        rotation_speed = random_rotation_speed_fast(rng);
                                    }
                                    Command::FlashPattern(attack_pattern, attack_flash) => {
                                        pattern = attack_pattern;
                                        flash = attack_flash;
                                        if flash == Flash::Strike && pattern[(player_i, player_o)] {
                                            break 'process GameResult::Death;
                                        }
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
                GameResult::Playing
            };
            let (play_state, draw_flashes) = match game_result {
                GameResult::Playing => (
                    PlayState::Playing {
                        commands,
                        curr_attack,
                        curr_transform,
                    },
                    speaker.beat_fraction() < 0.5,
                ),
                GameResult::Death => {
                    rotation_speed = random_rotation_speed_slow(rng);
                    speaker.play_sound(&sounds.death).unwrap();
                    speaker.stop_music();
                    speaker.restart_clock();
                    (
                        transition_state(PlayState::Death {
                            commands: VecDeque::new(),
                            attack: curr_attack,
                            original_transform: curr_transform,
                        }),
                        true,
                    )
                }
            };
            GameState {
                play_state,
                player,
                rotation,
                rotation_speed,
                pattern,
                flash,
                draw_flashes,
                tick,
                high_score,
            }
        }
        PlayState::Death {
            mut commands,
            attack,
            original_transform,
        } => {
            let mut pattern = gs.pattern;
            let mut flash = gs.flash;
            let mut draw_flashes = gs.draw_flashes;
            let mut rotation_speed = gs.rotation_speed;
            if is_key_pressed(KeyCode::Space) {
                new_game(rng, gs.high_score)
            } else {
                if matches!(speaker.tick(8), Tick::Beat(_)) {
                    draw_flashes = speaker.beat_fraction() < 0.5;
                }
                if let Some(attack) = &attack {
                    while let Some(tick) = speaker.process_tick(8) {
                        match tick {
                            Tick::Countdown(0..7) => {}
                            Tick::Countdown(7) => draw_flashes = false,
                            Tick::Countdown(_) => unreachable!(),
                            Tick::Beat(_) => {
                                if commands.is_empty() {
                                    let new_transform = attack.transform.construct(rng, gs.player);
                                    let transform = Transform {
                                        origin: new_transform.origin,
                                        transpose: original_transform.transpose,
                                        mirror: original_transform.mirror,
                                    };
                                    attack.clone().enqueue(&mut commands, transform);
                                }
                                loop {
                                    let Some(command) = commands.pop_front() else {
                                        break;
                                    };
                                    match command {
                                        Command::NewAttack(_, _) => {
                                            rotation_speed = random_rotation_speed_slow(rng);
                                        }
                                        Command::FlashPattern(attack_pattern, attack_flash) => {
                                            pattern = attack_pattern;
                                            flash = attack_flash;
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                GameState {
                    play_state: PlayState::Death {
                        commands,
                        attack,
                        original_transform,
                    },
                    rotation_speed,
                    pattern,
                    flash,
                    draw_flashes,
                    ..gs
                }
            }
        }
        PlayState::Transition(play_state) => {
            speaker.restart_clock();
            GameState {
                play_state: *play_state,
                ..gs
            }
        }
    };
    let frame_time = get_frame_time();
    let (in_rotation, out_rotation) = gs.rotation;
    let (in_rotation_speed, out_rotation_speed) = gs.rotation_speed;
    let rotation = (
        (in_rotation + in_rotation_speed * frame_time).rem_euclid(360.0),
        (out_rotation + out_rotation_speed * frame_time).rem_euclid(360.0),
    );
    GameState { rotation, ..gs }
}

fn draw<const N: usize>(screen: &Screen<N>, game_state: &GameState<N>, level: &Level<N>) {
    let GameState {
        play_state,
        player,
        rotation,
        rotation_speed: _,
        pattern,
        flash,
        draw_flashes,
        tick,
        high_score,
    } = game_state;
    clear_background(WHITE);
    match play_state {
        PlayState::Transition(_) => screen.flash(),
        _ => {
            screen.draw_playfield(
                pattern,
                *flash,
                *draw_flashes,
                *player,
                *rotation,
                &level.colors,
            );
            let tick_text = match tick {
                Tick::Countdown(tick @ 0..3) => format!("({})", 3 - tick),
                Tick::Countdown(3) => "GO".into(),
                Tick::Countdown(_) => unreachable!(),
                Tick::Beat(beat) => beat.to_string(),
            };
            screen.draw_text(&tick_text, 1.0 / 8.0);
            screen.draw_text(&high_score.to_string(), -1.0 / 8.0);
        }
    }
}
