pub fn is_facing_right(angle: f64) -> bool {
    angle.cos() > 0.0
}

pub fn is_facing_up(angle: f64) -> bool {
    angle.sin() > 0.0
}

pub fn is_horizontal_angle(angle: f64) -> bool {
    if angle == 0.0 || angle == std::f64::consts::PI {
        true
    } else {
        false
    }
}

pub fn base(perp: f64, angle: f64) -> f64 {
    (perp / angle.tan()) as f64
}

pub fn perp(base: f64, angle: f64) -> f64 {
    (base as f64 * angle.tan()) as f64
}

pub fn dist(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    let x = (x2 - x1);
    let y = (y2 - y1);
    (x * x + y * y).sqrt()
}
