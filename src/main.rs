extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Scancode};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, RenderTarget, TextureCreator};

use std::{thread, time};

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

fn base(perp: i32, angle:f64) -> i32 {
    (perp as f64 / angle.tan()) as i32
}

fn perp(base: i32, angle:f64) -> i32 {
    (base as f64 * angle.tan()) as i32
}

fn dist(x1:i32, y1:i32, x2:i32, y2:i32) -> f64{
    let x: f64 = (x2 - x1) as f64;
    let y: f64 = (y2 - y1) as f64;

    (x*x + y*y).sqrt()

}


struct World {
    y: i32,
    x: i32,
    angle: f64,
    layout: Vec<Vec<i32>>,
    grid_size: i32,
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
            vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        ];
            World {
                x: 3 * 64 + 32,
                y: 3 * 64 + 32,
                angle: 60.0_f64.to_radians(),
                grid_size: 64,
                layout: layout,
                // FIXME: 320 is hardcoded.
                heights: vec![0; 320],
                colors:  vec![0; 320],
            }
    }

    fn get_grid_values(&self, x: i32, y: i32) -> (i32, i32) {
        let mut x = x;
        let mut y = y;
        if x < 0 {
            x = x - self.grid_size;
        }
        if y < 0 {
            y = y - self.grid_size;
        }

        ( x / self.grid_size , y / self.grid_size )
    }

    fn is_wall_grid(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 {
            return true;
        }

        if (x as usize >= self.layout[0].len() || y as usize >= self.layout.len()) {
            return true;
        }

        self.layout[y as usize][x as usize] == 1
    }

    fn calc_horizontal_intersection(&self, x: i32, y:i32, angle: f64) -> (i32, i32) {

        if (1.0/angle.tan()).abs() > 100000000.0 {
            return (1000000, 10000000); // INF;
        }

        let mut Ay: i32 = if is_facing_up(angle) {
            (y/self.grid_size) * (self.grid_size) - 1
        } else {
            (y/self.grid_size) * (self.grid_size) + self.grid_size
        };

        let mut Ax: i32 = x + base(y-Ay, angle);
        let (mut Agx, mut Agy) = self.get_grid_values(Ax, Ay);


        let mut Ya:i32 = if is_facing_up(angle) {
            -(self.grid_size as i32)
        } else {
            (self.grid_size as i32)
        };

        let mut Xa:i32 = if y-Ay > 0 {
            base(self.grid_size, angle)
        } else {
            base(-1 * self.grid_size, angle)
        };

        while !self.is_wall_grid(Agx, Agy) {
            Ax += Xa;
            Ay += Ya;
            let grid_coordinates = self.get_grid_values(Ax, Ay);
            Agx = grid_coordinates.0;
            Agy = grid_coordinates.1;
        }
        (Ax, Ay)
    }

    fn calc_vertical_intersection(&self, x: i32, y:i32, angle: f64) -> (i32, i32) {

        if angle.tan().abs() > 100000000.0 {
            return (1000000, 10000000); // INF;
        }

        let mut Ax: i32 = if is_facing_right(angle) {
            (x/self.grid_size) * (self.grid_size) + self.grid_size
        } else {
            (x/self.grid_size) * (self.grid_size) -1
        };

        let mut Ay: i32 = y + perp(x-Ax, angle);
        let (mut Agx, mut Agy) = self.get_grid_values(Ax, Ay);

        let mut Xa:i32 = if is_facing_right(angle) {
            (self.grid_size as i32)
        } else {
            -1 * (self.grid_size as i32)
        };

        let mut Ya:i32 = if x-Ax > 0 {
            perp(self.grid_size, angle)
        } else {
            -1 * perp(self.grid_size, angle)
        };

        while !self.is_wall_grid(Agx, Agy) {
            Ax += Xa;
            Ay += Ya;
            let grid_coordinates = self.get_grid_values(Ax, Ay);
            Agx = grid_coordinates.0;
            Agy = grid_coordinates.1;
        }
        (Ax, Ay)
    }

    fn update_heights(&mut self) {
        let pi_by_3 = std::f64::consts::PI / 3.0;
        let pi_by_2 = std::f64::consts::PI / 2.0;


        let x = self.x;
        let y = self.y;

        for i in 0..320 {
            let angle = self.angle + pi_by_3 * (i as f64 / 320.0);
            let (hx, hy) = self.calc_horizontal_intersection(x, y, angle);
            let (vx, vy) = self.calc_vertical_intersection(x, y, angle);

            let hd = dist(x, y, hx, hy);
            let vd = dist(x, y, vx, vy);

            let mut d = hd.min(vd);

            if hd > vd {
                self.colors[i] = 1;
            } else {
                self.colors[i] = 0;
            }

            // Correct for fish eye.
            let beta = (self.angle + pi_by_3 / 2.0) - angle;

            d = d * beta.cos();

            let h = 64.0 / d * 255.0;

            self.heights[i] = h as i32;


        }
    }

}

fn main() {

    let width = 320;
    let height = 440;

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("Rust Ray Casting", width, height)
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
                Event::Quit {..} | Event::KeyDown {keycode: Some(Keycode::Escape), ..} => {
                    break 'running
                }
                Event::KeyDown {keycode: Some(Keycode::Left), ..} => {
                    w.x -= 1;
                }
                Event::KeyDown {keycode: Some(Keycode::Right), ..} => {
                    w.x += 1;
                }
                Event::KeyDown {keycode: Some(Keycode::Up), ..} => {
                    w.y -= 1;
                }
                Event::KeyDown {keycode: Some(Keycode::Down), ..} => {
                    w.y += 1;
                }
                Event::KeyDown {keycode: Some(Keycode::Q), ..} => {
                    w.angle += std::f64::consts::PI / 360.0;
                }
                Event::KeyDown {keycode: Some(Keycode::E), ..} => {
                    w.angle -= std::f64::consts::PI / 360.0;
                }
                _ => {}
            }
        }

        w.update_heights();
        canvas.set_draw_color(Color::RGB(128,128,128));
        canvas.fill_rect(
            Rect::new(0, 0, 320, 220)    
            );
        canvas.set_draw_color(Color::RGB(68,68,68));
        canvas.fill_rect(
            Rect::new(0, 220, 320, 220)    
            );


        for i in 0..320 {
            let h = w.heights[i];

            let rect = Rect::new(
                319 - i as i32,
                220 - (h / 2),
                1,
                h as u32
                );


            if (w.colors[i] == 0) {
                canvas.set_draw_color(Color::RGB(200,0,0));
            } else {
                canvas.set_draw_color(Color::RGB(255,0,0));
            }
            canvas.fill_rect(rect);

        }

        canvas.present();

        let ten_millis = time::Duration::from_millis(100);
        thread::sleep(ten_millis);


    }

}

