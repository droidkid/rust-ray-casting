extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Scancode};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, RenderTarget, TextureCreator};

use std::{thread, time};

const INF: f64 = 1e10;
const INF_PAIR: (f64, f64) = (INF, INF);

fn is_facing_right(angle: f64) -> bool {
    angle.cos() > 0.0
}

fn is_facing_up(angle: f64) -> bool {
    angle.sin() > 0.0
}

fn is_horizontal_angle(angle: f64) -> bool {
    if angle == 0.0 || angle == std::f64::consts::PI {
        true
    } else {
        false
    }
}

fn base(perp: f64, angle: f64) -> f64 {
    (perp / angle.tan()) as f64
}

fn perp(base: f64, angle: f64) -> f64 {
    (base as f64 * angle.tan()) as f64
}

fn dist(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    let x = (x2 - x1);
    let y = (y2 - y1);
    (x * x + y * y).sqrt()
}

struct World {
    x: f64,
    y: f64,
    angle: f64,
    fov_angle: f64,
    projection_width: u32,
    projection_dist: u32,
    layout: Vec<Vec<i32>>,
    minimap: Vec<Vec<i32>>,
    grid_size: f64,
    heights: Vec<i32>,
    colors: Vec<i32>,
}

impl World {
    fn new() -> World {
        let layout = vec![
            vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            vec![1, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 1],
            vec![1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1],
            vec![1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1],
            vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        ];

        let minimap = vec![
            vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            vec![1, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 1],
            vec![1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1],
            vec![1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1],
            vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        ];

        World {
            x: 224.0,
            y: 481.0,
            angle: 60.0_f64.to_radians(),
            fov_angle: 60.0_f64.to_radians(),
            projection_width: 800,
            projection_dist: 255,
            grid_size: 64.0,
            layout: layout,
            minimap: minimap,
            heights: vec![0; 800],
            colors: vec![0; 800],
        }
    }

    fn convert_to_grid(&self, x: f64, y: f64) -> (i32, i32) {
        let mut x = x;
        let mut y = y;

        // Decrement negative numbers.
        // Because division rounds to 0.
        // We need it to round to -infinity.
        if x < 0.0 {
            x = x - self.grid_size;
        }
        if y < 0.0 {
            y = y - self.grid_size;
        }

        ( (x/self.grid_size) as i32, (y/self.grid_size) as i32)
    }

    fn is_wall_grid(&self, x: f64, y: f64) -> bool {
        let (mut xg, mut yg) = self.convert_to_grid(x, y);

        if xg < 0 || yg < 0 {
            return true;
        }
        if (xg as usize >= self.layout[0].len() || yg as usize >= self.layout.len()) {
            return true;
        }
        self.layout[yg as usize][xg as usize] == 1
    }

    fn calc_horizontal_intersection(&mut self, x: f64, y: f64, angle: f64) -> (f64, f64) {
        if (1.0 / angle.tan()).abs() > INF {
            return INF_PAIR;
        }

        let mut ny: f64 = if is_facing_up(angle) {
            (y as i32 / self.grid_size as i32) as f64 * (self.grid_size) - 1.0
        } else {
            (y as i32 / self.grid_size as i32) as f64 * (self.grid_size) + self.grid_size
        };
        let mut nx: f64 = x + base(y - ny, angle);

        let mut dy: f64 = if is_facing_up(angle) {
            -self.grid_size
        } else {
            self.grid_size
        };

        let mut dx: f64 = if y - ny > 0.0 {
            base(self.grid_size, angle)
        } else {
            base(-self.grid_size, angle)
        };

        while !self.is_wall_grid(nx, ny) {
            let (ngx, ngy) = self.convert_to_grid(nx, ny);
            self.minimap[ngy as usize][ngx as usize] = 2;
            nx += dx;
            ny += dy;
        }
        (nx, ny)
    }

    fn calc_vertical_intersection(&mut self, x: f64, y: f64, angle: f64) -> (f64, f64) {
        if angle.tan().abs() > INF {
            return INF_PAIR;
        }

        let mut nx: f64 = if is_facing_right(angle) {
            (x as i32 / self.grid_size as i32) as f64 * (self.grid_size) + self.grid_size
        } else {
            (x as i32 / self.grid_size as i32) as f64 * (self.grid_size) - 1.0
        };
        let mut ny: f64 = y + perp(x - nx , angle);

        let mut dx: f64 = if is_facing_right(angle) {
            self.grid_size
        } else {
            (-self.grid_size)
        };

        let mut dy: f64 = if x - nx > 0.0 {
            perp(self.grid_size, angle)
        } else {
            perp(-1.0 * self.grid_size, angle)
        };

        while !self.is_wall_grid(nx, ny) {
            let (ngx, ngy) = self.convert_to_grid(nx, ny);
            self.minimap[ngy as usize][ngx as usize] = 2;
            nx += dx;
            ny += dy;
        }
        (nx, ny)
    }

    fn reset_minimap(&mut self) {
        for i in 0..self.layout.len() {
            for j in 0..self.layout[i].len() {
                self.minimap[i][j] = self.layout[i][j];
            }
        }
    }

    fn update_heights(&mut self) {
        let pi_by_3 = std::f64::consts::PI / 3.0;
        let pi_by_2 = std::f64::consts::PI / 2.0;

        let x = self.x as f64;
        let y = self.y as f64;


        for i in 0..self.projection_width {
            let angle = self.angle - self.fov_angle/2.0 + self.fov_angle * (i as f64 / self.projection_width as f64);
            let (hx, hy) = self.calc_horizontal_intersection(x, y, angle);
            let hd = dist(x, y, hx, hy);

            let (vx, vy) = self.calc_vertical_intersection(x, y, angle);
            let vd = dist(x, y, vx, vy);

            let mut d = hd.min(vd);
            self.colors[i as usize] = 255 * 100 / d as i32;

            let beta = (self.angle - self.fov_angle/2.0 + pi_by_3 / 2.0) - (angle);
            d = d * beta.cos();

            let h = self.grid_size / d * self.projection_width as f64;
            self.heights[i as usize] = h as i32;
        }
    }
}

fn main() {
    let width = 800;
    let height = 600;

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Rust Ray Casting", width, height)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut w = World::new();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    ..
                } => {
                    w.x -= 3.0 * w.angle.sin();
                    w.y -= 3.0 * w.angle.cos();
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => {
                    w.x += 3.0 * w.angle.sin();
                    w.y += 3.0 * w.angle.cos();
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Up),
                    ..
                } => {
                    w.y -= 3.0 * w.angle.sin();
                    w.x += 3.0 * w.angle.cos();
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Down),
                    ..
                } => {
                    w.y += 3.0 * w.angle.sin();
                    w.x -= 3.0 * w.angle.cos();
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Q),
                    ..
                } => {
                    w.angle += std::f64::consts::PI / 120.0;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::E),
                    ..
                } => {
                    w.angle -= std::f64::consts::PI / 120.0;
                }
                _ => {}
            }
        }

        w.update_heights();

        canvas.set_draw_color(Color::RGB(128, 128, 128));
        canvas.fill_rect(Rect::new(0, 0, 800, 300));
        canvas.set_draw_color(Color::RGB(68, 68, 68));
        canvas.fill_rect(Rect::new(0, 300, 800, 300));

        for i in 0..w.projection_width {
            let h = w.heights[i as usize];
            let rect = Rect::new(799 - i as i32, 300 - (h / 2), 1, h as u32);
            canvas.set_draw_color(Color::RGB(w.colors[i as usize] as u8, 0, 0));
            canvas.fill_rect(rect);
        }

        let mut mx = 500;
        let mut my = 0;

        for i in 0..w.minimap.len() {
            for j in 0..w.minimap[i].len() {
                let rect = Rect::new(mx, my, 5, 5);

                if w.minimap[i][j] == 0 {
                    canvas.set_draw_color(Color::RGB(64, 64, 64));
                } else if w.minimap[i][j] == 1 {
                    canvas.set_draw_color(Color::RGB(120, 0, 0));
                } else if w.minimap[i][j] == 2 {
                    canvas.set_draw_color(Color::RGB(220, 220, 220));
                }
                mx += 5;
                canvas.fill_rect(rect);
            }
            mx = 500;
            my += 5;
        }
        canvas.present();
        w.reset_minimap();

        let ten_millis = time::Duration::from_millis(10);
        thread::sleep(ten_millis);
    }
}
