use macroquad::{
    color::{Color, WHITE},
    shapes::{draw_poly, draw_poly_lines, draw_rectangle},
    text::{draw_text, get_text_center},
};

use crate::command::Flash;
use crate::level::LevelColors;
use crate::playfield::{Coord, Pattern};

#[derive(Debug)]
pub struct Screen<const N: usize> {
    playfield_size: f32,
}

#[derive(Clone, Copy, Debug)]
enum DrawMode {
    Fill,
    Stroke(f32),
}

impl<const N: usize> Screen<N> {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            playfield_size: width.min(height),
        }
    }

    fn fill_background(&self, color: Color) {
        draw_rectangle(0.0, 0.0, self.playfield_size, self.playfield_size, color);
    }

    fn draw_polygon(
        &self,
        (x, y): (f32, f32),
        radius: f32,
        draw_mode: DrawMode,
        rotation: f32,
        color: Color,
    ) {
        let thickness = match draw_mode {
            DrawMode::Fill => None,
            DrawMode::Stroke(thickness) => Some(thickness),
        };
        let scale = self.playfield_size / 2.0;
        let x = (x + 1.0) * scale;
        let y = (-y + 1.0) * scale;
        let sides = N as u8;
        let radius = (radius - thickness.unwrap_or(0.0) / 2.0) * scale;
        let rotation = rotation - 90.0;
        if let Some(thickness) = thickness {
            let thickness = thickness * scale;
            draw_poly_lines(x, y, sides, radius, rotation, thickness, color);
        } else {
            draw_poly(x, y, sides, radius, rotation, color);
        }
    }

    pub fn draw_playfield(
        &self,
        pattern: &Pattern<N>,
        flash: Flash,
        draw_flashes: bool,
        player: (Coord<N>, Coord<N>),
        rotation: (f32, f32),
        colors: &LevelColors<N>,
    ) {
        fn orbit<const N: usize>(
            n: Coord<N>,
            (x, y): (f32, f32),
            radius: f32,
            rotation: f32,
        ) -> (f32, f32) {
            let t = ((90.0 - rotation) - 360.0 * (n.inner() as f32) / (N as f32)).to_radians();
            (x + t.cos() * radius, y + t.sin() * radius)
        }

        let (in_rotation, out_rotation) = rotation;
        self.fill_background(colors.background);
        let r = 1.0 / (180.0 / (N as f32)).to_radians().sin();
        let base_radius = 1.0 / (1.0 + r + r + r * r);
        let medium_radius = base_radius * r;
        let large_radius = (base_radius + medium_radius) * r;
        let small_radius = medium_radius * (medium_radius / large_radius);
        let line_thickness = small_radius / 3.0;
        self.draw_polygon(
            (0.0, 0.0),
            large_radius,
            DrawMode::Stroke(line_thickness),
            out_rotation,
            colors.out_ring,
        );
        for o in Coord::iter_all() {
            let (x, y) = orbit(o, (0.0, 0.0), large_radius, out_rotation);
            self.draw_polygon(
                (x, y),
                medium_radius,
                DrawMode::Stroke(line_thickness),
                in_rotation,
                colors.main[o.inner()],
            );
            for i in Coord::iter_all() {
                let (x, y) = orbit(i, (x, y), medium_radius, in_rotation);
                let regular_color = colors.main[i.inner()];
                let draw_tile = |scale, color| {
                    self.draw_polygon(
                        (x, y),
                        small_radius * scale,
                        DrawMode::Fill,
                        in_rotation,
                        color,
                    );
                };
                match (draw_flashes && pattern[(i, o)], flash) {
                    (false, _) => draw_tile(1.0, regular_color),
                    (true, Flash::Warn) => {
                        draw_tile(4.0 / 3.0, colors.flash);
                        draw_tile(1.0, regular_color);
                    }
                    (true, Flash::Strike) => draw_tile(4.5 / 3.0, colors.flash),
                }
                if (i, o) == player {
                    draw_tile(2.0 / 3.0, colors.player);
                }
            }
        }
    }

    pub fn draw_text(&self, text: &str, y: f32) {
        let x = self.playfield_size / 2.0;
        let y = (-y + 1.0) * self.playfield_size / 2.0;
        let font_size = self.playfield_size / 8.0;
        let center = get_text_center(text, None, font_size as u16, 1.0, 0.0);
        draw_text(text, x - center.x, y - center.y, font_size, WHITE);
    }

    pub fn flash(&self) {
        self.fill_background(WHITE);
    }
}
