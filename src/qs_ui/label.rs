use super::{Widget, Picture};
use quicksilver::lifecycle::{Window};
use quicksilver::geom::{Rectangle, Shape, Vector};
use quicksilver::graphics::{Font, FontStyle};
use quicksilver::Result;

pub struct Label {
    pub picture: Picture,
}

impl Label {
    pub fn new<S>(text: S, font: &Font, style: FontStyle) -> Result<Label> where S: Into<String> {
        let text = text.into();
        let image = font.render(&text, &style)?;

        Ok(Label {
            picture: Picture::new(image),
        })
    }

    pub fn with_area(&mut self, area: Rectangle) -> &mut Self {
        self.picture.with_area(area);
        self
    }

    pub fn with_pos(&mut self, pos: Vector) -> &mut Self {
        self.picture.with_pos(pos);
        self
    }

    pub fn with_center(&mut self, pos: Vector) -> &mut Self {
        self.picture.with_center(pos);
        self
    }
}

impl Widget for Label {
    fn area(&self) -> Rectangle {
        self.picture.area()
    }

    fn draw(&mut self, window: &mut Window) -> Result<()> {
        self.picture.draw(window)
    }
}