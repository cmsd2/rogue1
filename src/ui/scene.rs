use tui::{Frame};
use tui::layout::Rect;
use tui::backend::Backend;
use tui::widgets::{Widget, Block, Borders};
use crate::ui::level_view::LevelView;
use crate::ui::cursor::Cursor;
use crate::ui::path::Path;
use crate::glfw_system::RenderContext;
use crate::ecs::Position;

pub enum Scene {
    Blank,
    Text {
        title: String,
        cursor: Option<Position>,
        path: Option<Vec<Position>>,
    }
}

impl Default for Scene {
    fn default() -> Self {
        Scene::Blank
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
            Scene::Text { title, cursor, path } => {
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .render(f, size);

                let inner = size.inner(1);
                let mut p = LevelView::new(render_context.level, render_context.entities, render_context.characters, render_context.positions)
                    .size(inner);
                p.render(f, inner);

                if let Some(position) = cursor {
                    Cursor::new(position.to_owned())
                        .render(f, inner);
                }

                if let Some(path) = path {
                    Path::new(path)
                        .render(f, inner);
                }
            }
        }
    }
}