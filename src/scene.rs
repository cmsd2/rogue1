use tui::{Frame};
use tui::layout::Rect;
use tui::backend::Backend;
use tui::widgets::{Widget, Block, Borders};
use crate::level_view::LevelView;
use crate::glfw_system::RenderContext;

pub enum Scene {
    Blank,
    Text(String),
}

impl Default for Scene {
    fn default() -> Self {
        Scene::Text("hello, world!".to_string())
    }
}

impl Scene {
    pub fn render<'a, 'b, B>(&mut self, f: &mut Frame<B>, size: Rect, render_context: RenderContext<'a,'b>) where B: Backend {
        match self {
            Scene::Blank => {
                Block::default()
                    .title("Blank")
                    .borders(Borders::ALL)
                    .render(f, size);
            },
            Scene::Text(_s) => {
                Block::default()
                    .title("Text")
                    .borders(Borders::ALL)
                    .render(f, size);

                let inner = size.inner(1);
                let mut p = LevelView::new(render_context.level, render_context.entities, render_context.characters, render_context.positions)
                    .size(inner);
                p.render(f, inner);
            }
        }
    }
}