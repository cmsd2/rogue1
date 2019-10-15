use quicksilver::graphics::Color as QsColor;
use tint::Color as TintColor;
use crate::tween::{Tweenable, Tweener};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Color {
    hue: Hue,
    value: u8,
}

impl Color {
    pub fn new(hue: Hue, value: u8) -> Self {
        Color {
            hue: hue,
            value: value,
        }
    }

    pub fn darker(&self) -> Color {
        Color::new(self.hue, self.value / 2)
    }

    pub fn qs_color(&self, palette: &Palette) -> QsColor {
        palette.qs_color(self.hue, self.value)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Hue {
    Red = 0,
    Orange = 1,
    Yellow = 2,
    Lime = 3,
    Green = 4,
    Cyan = 5,
    Azure = 6,
    Blue = 7,
    Indigo = 8,
    Violet = 9,
    Pink = 10,
    White = 11,
}

pub struct Palette {
    colors: ColorTable,
}

impl Palette {
    pub fn new() -> Palette {
        let mut colors = ColorTable::new();
        colors.hv_table(12, 12);
        Palette {
            colors: colors,
        }
    }

    pub fn qs_color(&self, hue: Hue, value: u8) -> QsColor {
        self.colors.qs_color(((255 - value) as f32 * 11.0 / 255.0) as usize, hue as usize)
    }

    pub fn color(&self, hue: Hue, value: u8) -> Color {
        Color::new(hue, value)
    }
}

pub struct ColorTable {
    rows: usize,
    cols: usize,
    colors: Vec<TintColor>,
}

impl ColorTable {
    pub fn new() -> ColorTable {
        ColorTable {
            rows: 0,
            cols: 0,
            colors: vec![],
        }
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn qs_color(&self, col: usize, row: usize) -> QsColor {
        let c = self.tint_color(col, row);
        QsColor { r: c.red as f32, g: c.green as f32, b: c.blue as f32, a: c.alpha as f32 }
    }

    pub fn tint_color(&self, col: usize, row: usize) -> TintColor {
        self.colors[col + row * self.cols]
    }

    pub fn hv_table(&mut self, shades: usize, hues: usize) {
        let hue_t = Tweener::Increment { to: 1.0, step: 1.0 / (hues - 1) as f64 };
        let value_t = Tweener::Decrement { to: 0.0, step: 1.0 / (shades - 1) as f64 };

        println!("shades: {} hues: {}", shades, hues);

        let mut hue = 0.0;
        for _row in 1..hues {
            let mut value = 1.0;
            for _col in 0..shades {
                self.colors.push(TintColor::new(hue * 360.0, 1.0, value, 1.0).from_hsv());
                value.tween(&value_t);    
            }
            hue.tween(&hue_t);
        }

        let mut value = 1.0;
        for _col in 0..shades {
            self.colors.push(TintColor::new(0.0, 0.0, value, 1.0).from_hsv());
            value.tween(&value_t);
        }

        println!("colors: {}", self.colors.len());

        self.cols = shades;
        self.rows = hues;
    }
}
