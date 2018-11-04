extern crate sdl2;

use sdl2::event::Event;
use sdl2::image::{LoadTexture, INIT_JPG, INIT_PNG};
use sdl2::keyboard::{Keycode, Scancode};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, RenderTarget, TextureCreator};
use std::path::Path;

use std::{thread, time};

mod math_util;

const INF: f64 = 1e12;
const INF_PAIR: (f64, f64) = (INF, INF);

struct World {
    x: f64,
    y: f64,
    ray_angle: f64,
    fov_angle: f64,
    projection_width: u32,
    projection_dist: u32,
    layout: Vec<Vec<i32>>,
    grid_size: f64,
    heights: Vec<i32>,
    edge_dist: Vec<i32>,
    wall_orient: Vec<i32>,
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


        World {
            x: 224.0,
            y: 481.0,
            ray_angle: 60.0_f64.to_radians(),
            fov_angle: 60.0_f64.to_radians(),
            projection_width: 800,
            projection_dist: 255,
            grid_size: 64.0,
            layout: layout,
            heights: vec![0; 800],
            edge_dist: vec![0; 800],
            wall_orient: vec![0; 800],
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

        ((x / self.grid_size) as i32, (y / self.grid_size) as i32)
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

        let mut ny: f64 = if math_util::is_facing_up(angle) {
            (y as i32 / self.grid_size as i32) as f64 * (self.grid_size) - 1.0
        } else {
            (y as i32 / self.grid_size as i32) as f64 * (self.grid_size) + self.grid_size
        };
        let mut nx: f64 = x + math_util::base(y - ny, angle);

        let mut dy: f64 = if math_util::is_facing_up(angle) {
            -self.grid_size
        } else {
            self.grid_size
        };

        let mut dx: f64 = if y - ny > 0.0 {
            math_util::base(self.grid_size, angle)
        } else {
            math_util::base(-self.grid_size, angle)
        };

        while !self.is_wall_grid(nx, ny) {
            let (ngx, ngy) = self.convert_to_grid(nx, ny);
            nx += dx;
            ny += dy;
        }
        (nx, ny)
    }

    fn calc_vertical_intersection(&mut self, x: f64, y: f64, angle: f64) -> (f64, f64) {
        if angle.tan().abs() > INF {
            return INF_PAIR;
        }

        let mut nx: f64 = if math_util::is_facing_right(angle) {
            (x as i32 / self.grid_size as i32) as f64 * (self.grid_size) + self.grid_size
        } else {
            (x as i32 / self.grid_size as i32) as f64 * (self.grid_size) - 1.0
        };
        let mut ny: f64 = y + math_util::perp(x - nx, angle);

        let mut dx: f64 = if math_util::is_facing_right(angle) {
            self.grid_size
        } else {
            -self.grid_size
        };

        let mut dy: f64 = if x - nx > 0.0 {
            math_util::perp(self.grid_size, angle)
        } else {
            math_util::perp(-1.0 * self.grid_size, angle)
        };

        while !self.is_wall_grid(nx, ny) {
            let (ngx, ngy) = self.convert_to_grid(nx, ny);
            nx += dx;
            ny += dy;
        }
        (nx, ny)
    }

    fn update_heights(&mut self) {
        let x = self.x as f64;
        let y = self.y as f64;

        for i in 0..self.projection_width {
            let angle = self.ray_angle - self.fov_angle / 2.0
                + self.fov_angle * (i as f64 / self.projection_width as f64);
            let (hx, hy) = self.calc_horizontal_intersection(x, y, angle);
            let hd = math_util::dist(x, y, hx, hy);

            let (vx, vy) = self.calc_vertical_intersection(x, y, angle);
            let vd = math_util::dist(x, y, vx, vy);

            let mut d = hd.min(vd);

            let beta = (self.ray_angle - angle);
            d = d * beta.cos();

            let h = self.grid_size / d * self.projection_width as f64;
            self.heights[i as usize] = h as i32;

            if hd < vd {
                self.wall_orient[i as usize] = 1;
                self.edge_dist[i as usize] = (hx as i32) % (self.grid_size as i32)
            } else if hd > vd {
                self.wall_orient[i as usize] = 2;
                self.edge_dist[i as usize] = (vy as i32) % (self.grid_size as i32)
            };
        }
    }

    fn draw(&mut self, width:i32, height:i32) {

    }
}

fn main() {
    let width = 800;
    let height = 600;

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let _image_context = sdl2::image::init(INIT_PNG | INIT_JPG).unwrap();

    let window = video_subsystem
        .window("Rust Ray Casting", width, height)
        .position_centered()
        .borderless()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().software().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let texture_creator = canvas.texture_creator();

    sdl_context.mouse().show_cursor(false);

    let mut w = World::new();

    let wall1_texture_path = Path::new("res/bg_wood_light.png");
    let texture1 = texture_creator.load_texture(&wall1_texture_path).unwrap();
    let wall2_texture_path = Path::new("res/bg_wood_light.png");
    let texture2 = texture_creator.load_texture(&wall2_texture_path).unwrap();
    let rifle_texture_path = Path::new("res/rifle.png");
    let rifleTexture = texture_creator.load_texture(&rifle_texture_path).unwrap();
    let crosshair_path = Path::new("res/crosshair_red_small.png");
    let crosshair = texture_creator.load_texture(&crosshair_path).unwrap();

    let mut mouse_x: i32 = 0;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::A),
                    ..
                } => {
                    w.x -= 6.0 * w.ray_angle.sin();
                    w.y -= 6.0 * w.ray_angle.cos();
                }
                Event::KeyDown {
                    keycode: Some(Keycode::D),
                    ..
                } => {
                    w.x += 6.0 * w.ray_angle.sin();
                    w.y += 6.0 * w.ray_angle.cos();
                }
                Event::KeyDown {
                    keycode: Some(Keycode::W),
                    ..
                } => {
                    w.y -= 6.0 * w.ray_angle.sin();
                    w.x += 6.0 * w.ray_angle.cos();
                }
                Event::KeyDown {
                    keycode: Some(Keycode::S),
                    ..
                } => {
                    w.y += 6.0 * w.ray_angle.sin();
                    w.x -= 6.0 * w.ray_angle.cos();
                }
                Event::MouseMotion { x, .. } => {
                    let dx = mouse_x - x;
                    mouse_x = x;

                    w.ray_angle += (dx as f64 / 300.0) * std::f64::consts::PI;
                }

                _ => {}
            }
        }

        w.update_heights();

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.fill_rect(Rect::new(0, 0, 800, 300));
        canvas.set_draw_color(Color::RGB(168, 100, 100));
        canvas.fill_rect(Rect::new(0, 300, 800, 300));

        for i in 0..w.projection_width {
            let h = w.heights[i as usize];
            let dest_rect = Rect::new(799 - i as i32, 300 - (h / 2), 1, h as u32);
            let src_rect = Rect::new(w.edge_dist[i as usize] * 4, 0, 1, 512);

            if w.wall_orient[i as usize] == 2 {
                canvas.copy(&texture1, src_rect, dest_rect);
            }
            if w.wall_orient[i as usize] == 1 {
                canvas.copy(&texture2, src_rect, dest_rect);
            }
        }

        let rifle_dest_rect = Rect::new(400, 350, 280, 400);
        let rifle_src_rect = Rect::new(0, 0, 142, 130);
        canvas.copy(&rifleTexture, rifle_src_rect, rifle_dest_rect);

        let mut mx = 500;
        let mut my = 0;

        canvas.present();

        let ten_millis = time::Duration::from_millis(10);
        thread::sleep(ten_millis);
    }
}
