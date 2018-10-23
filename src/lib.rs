struct World {
    layout: Vec<Vec<i32>>,
    grid_size: i32,
}

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

fn base(perp: i32, angle: f64) -> i32 {
    (perp as f64 / angle.tan()) as i32
}

fn perp(base: i32, angle: f64) -> i32 {
    (base as f64 * angle.tan()) as i32
}

impl World {
    fn new() -> World {
        let layout = vec![
            vec![1, 1, 1, 1, 1],
            vec![1, 0, 0, 0, 1],
            vec![1, 0, 0, 0, 1],
            vec![1, 0, 0, 0, 1],
            vec![1, 0, 0, 0, 1],
            vec![1, 1, 1, 1, 1],
        ];
        World {
            grid_size: 64,
            layout: layout,
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

        (x / self.grid_size, y / self.grid_size)
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

    fn calc_horizontal_intersection(&self, x: i32, y: i32, angle: f64) -> (i32, i32) {
        if (1.0 / angle.tan()).abs() > 100000000.0 {
            return (1000000, 10000000); // INF;
        }

        let mut Ay: i32 = if is_facing_up(angle) {
            (y / self.grid_size) * (self.grid_size) - 1
        } else {
            (y / self.grid_size) * (self.grid_size) + self.grid_size
        };

        let mut Ax: i32 = x + base(y - Ay, angle);
        let (mut Agx, mut Agy) = self.get_grid_values(Ax, Ay);

        let mut Ya: i32 = if is_facing_up(angle) {
            -(self.grid_size as i32)
        } else {
            (self.grid_size as i32)
        };

        let mut Xa: i32 = if is_facing_right(angle) {
            base(self.grid_size, angle)
        } else {
            -1 * base(self.grid_size, angle)
        };

        while !self.is_wall_grid(Agx, Agy) {
            Ax += Xa;
            Ay += Ya;
            let grid_coordinates = self.get_grid_values(Ax, Ay);
            Agx = grid_coordinates.0;
            Agy = grid_coordinates.1;

            if Agx < 0 || Agy < 0 {
                println!("INF");
                return (1000000, 1000000); // INF
            }
        }

        println!("{} {}", Ax, Ay);
        (Ax, Ay)
    }

    fn calc_vertical_intersection(&self, x: i32, y: i32, angle: f64) -> (i32, i32) {
        if angle.tan().abs() > 100000000.0 {
            return (1000000, 10000000); // INF;
        }

        // TODO: handle vertical line case.
        let mut Ax: i32 = if is_facing_right(angle) {
            (x / self.grid_size) * (self.grid_size) + self.grid_size
        } else {
            (x / self.grid_size) * (self.grid_size) - 1
        };

        let mut Ay: i32 = y + perp(x - Ax, angle);
        let (mut Agx, mut Agy) = self.get_grid_values(Ax, Ay);

        let mut Xa: i32 = if is_facing_right(angle) {
            (self.grid_size as i32)
        } else {
            -1 * (self.grid_size as i32)
        };

        let mut Ya: i32 = if is_facing_up(angle) {
            -1 * perp(self.grid_size, angle)
        } else {
            perp(self.grid_size, angle)
        };

        while !self.is_wall_grid(Agx, Agy) {
            Ax += Xa;
            Ay += Ya;
            let grid_coordinates = self.get_grid_values(Ax, Ay);
            Agx = grid_coordinates.0;
            Agy = grid_coordinates.1;

            if Agx < 0 || Agy < 0 {
                println!("INF");
                return (1000000, 1000000); // INF
            }
        }

        println!("{} {}", Ax, Ay);
        (Ax, Ay)
    }
}

/*
fn main() {
    let w = World::new();
    println!("Starting Test");

    let x = 96;
    let y = 224;

    for i in 0..360 {
        let pi = std::f64::consts::PI;
        let angle = (i as f64/360.0) * 2.0 * pi;
        println!("angle: {}", angle.to_degrees());
        let (hx, hy) = w.calc_horizontal_intersection(x, y, angle);
        let (vx, vy) = w.calc_vertical_intersection(x, y, angle);
    }

}
*/
