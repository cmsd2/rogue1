use quicksilver::lifecycle::Window;
use quicksilver::graphics::View;
use quicksilver::geom::{Rectangle, Transform};
use quicksilver::Result;

pub struct WithView(View);

impl WithView {
    pub fn execute<F>(self, window: &mut Window, f: F) -> Result<()> where F: FnOnce(&mut Window) -> Result<()> {
        match self {
            WithView(view) => {
                let old_view = window.view();
                window.set_view(view);
                let ret = f(window);
                window.set_view(old_view);
                ret
            }
        }
    }
}

pub struct WithClientView(Rectangle);

impl WithClientView {
    pub fn execute<F>(self, window: &mut Window, f: F) -> Result<()> where F: FnOnce(&mut Window) -> Result<()> {
        match self {
            WithClientView(rect) => {
                let transform = Transform::IDENTITY; // Transform::translate(rect.top_left());
                WithView(View::new_transformed(rect, transform)).execute(window, f)
            }
        }
    }
}

pub trait Widget {
    fn draw(&mut self, window: &mut Window) -> Result<()>;

    fn area(&self) -> Rectangle;
}