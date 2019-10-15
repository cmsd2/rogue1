use tui::layout::Rect;
use quicksilver::geom::{Rectangle, Vector};

#[derive(Debug, Clone, PartialEq)]
pub enum LayoutRect {
    Rect(Rect),
    Rectangle(Rectangle)
}

impl From<Rect> for LayoutRect {
    fn from(r: Rect) -> Self {
        LayoutRect::Rect(r)
    }
}

impl From<Rectangle> for LayoutRect {
    fn from(r: Rectangle) -> Self {
        LayoutRect::Rectangle(r)
    }
}

impl Into<Rectangle> for LayoutRect {
    fn into(self) -> Rectangle {
        match self {
            LayoutRect::Rect(r) => Rectangle::new(Vector::new(r.x, r.y), Vector::new(r.width, r.height)),
            LayoutRect::Rectangle(r) => r.to_owned()
        }
    }
}

impl Into<Rect> for LayoutRect {
    fn into(self) -> Rect {
        match self {
            LayoutRect::Rect(r) => r.to_owned(),
            LayoutRect::Rectangle(r) => Rect { x: r.x() as u16, y: r.y() as u16, width: r.width() as u16, height: r.height() as u16 },
        }
    }
}
