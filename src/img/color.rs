use crate::math::Vec3;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Color(pub Vec3);

impl Color {
    pub const BLACK: Color = Color::from_rgb(0., 0., 0.);
    pub const WHITE: Color = Color::from_rgb(1., 1., 1.);
    pub const RED: Color = Color::from_rgb(1., 0., 0.);
    pub const GREEN: Color = Color::from_rgb(0., 1., 0.);
    pub const BLUE: Color = Color::from_rgb(0., 0., 1.);

    pub const fn from_rgb(r: f64, g: f64, b: f64) -> Self {
        Self(Vec3::new(r, g, b))
    }

    pub const fn gray(brightness: f64) -> Self {
        Self::from_rgb(brightness, brightness, brightness)
    }

    pub const fn r(self) -> f64 {
        self.0.x
    }

    pub const fn g(self) -> f64 {
        self.0.y
    }

    pub const fn b(self) -> f64 {
        self.0.z
    }

    pub fn to_rgb_bytes(self) -> [u8; 3] {
        [
            to_percent_byte(self.r()),
            to_percent_byte(self.g()),
            to_percent_byte(self.b()),
        ]
    }

    pub fn reflect_on(self, surface: Color) -> Color {
        Color(Vec3::new(
            self.0.x * surface.0.x,
            self.0.y * surface.0.y,
            self.0.z * surface.0.z,
        ))
    }
}

impl From<Vec3> for Color {
    fn from(value: Vec3) -> Self {
        Self(value)
    }
}

impl From<Color> for Vec3 {
    fn from(value: Color) -> Self {
        value.0
    }
}

fn to_percent_byte(x: f64) -> u8 {
    (x * 256.).clamp(0., 255.).floor() as u8
}
