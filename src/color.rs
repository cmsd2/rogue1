use tui::style::Color;

#[derive(Copy, Clone, Debug)]
pub struct ColorMap {
    pub default_fg: [f32; 4],
}

impl Default for ColorMap {
    fn default() -> Self {
        let mut color_map = ColorMap {
            default_fg: [1.0; 4]
        };
        color_map.default_fg = color_map.lookup_tui(Color::Gray);
        color_map
    }
}

impl ColorMap {
    pub fn lookup_tui(&self, c: Color) -> [f32; 4] {
        let z = 0.0;
        let d = 0.4;
        let l = 0.7;
        let f = 1.0;
        match c {
            Color::Black =>        [z, z, z, 1.0],
            Color::Red =>          [l, z, z, 1.0],
            Color::Green =>        [z, l, z, 1.0],
            Color::Yellow =>       [l, l, z, 1.0],
            Color::Blue =>         [z, z, l, 1.0],
            Color::Magenta =>      [l, z, l, 1.0],
            Color::Cyan =>         [z, l, l, 1.0],
            Color::Gray =>         [l, l, l, 1.0],
            Color::DarkGray =>     [d, d, d, 1.0],
            Color::LightRed =>     [f, z, z, 1.0],
            Color::LightGreen =>   [z, f, z, 1.0],
            Color::LightYellow =>  [f, f, z, 1.0],
            Color::LightBlue =>    [z, z, f, 1.0],
            Color::LightMagenta => [f, z, f, 1.0],
            Color::LightCyan =>    [z, f, f, 1.0],
            Color::White =>        [f, f, f, 1.0],
            Color::Rgb(r,g,b) =>   [r as f32 / 256.0, g as f32 / 256.0, b as f32 / 256.0, 1.0],
            Color::Indexed(_i) => unimplemented!(),
            Color::Reset => self.default_fg,  
        }
    }
}