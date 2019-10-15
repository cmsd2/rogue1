use super::widget::Widget;
use quicksilver::lifecycle::{Window};
use quicksilver::geom::{Rectangle, Shape, Vector};
use quicksilver::graphics::{Background, Image, Font, FontStyle};
use quicksilver::Result;

pub struct Picture {
    pub image: Image,
    area: Rectangle,
    pos: Vector,
}

impl Picture {
    pub fn new(image: Image) -> Picture {
        Picture {
            area: image.area(),
            image: image,
            pos: Vector::ZERO,
        }
    }

    pub fn with_area(&mut self, area: Rectangle) -> &mut Self {
        self.area = area;
        self
    }

    pub fn with_pos(&mut self, pos: Vector) -> &mut Self {
        self.pos = pos;
        self
    }

    pub fn with_center(&mut self, pos: Vector) -> &mut Self {
        self.pos = pos - self.area().center();
        self
    }
}

impl Widget for Picture {
    fn area(&self) -> Rectangle {
        self.area.clone()
    }

    fn draw(&mut self, window: &mut Window) -> Result<()> {
        window.draw(
            &self.area.translate(self.pos),
            Background::Img(&self.image),
            );
        Ok(())
    }
}