use pathfinding::prelude::*;
use super::level::Level;
use super::ecs::Position;

pub struct PathFinder<'a> {
    pub level: &'a Level,
}

impl <'a> PathFinder<'a> {
    pub fn new(level: &'a Level) -> Self {
        PathFinder {
            level: level,
        }
    }

    fn move_cost(&self, _src: &Position, dst: &Position, target: &Position) -> u32 {
        let level_cell = self.level.get(dst.x as u16, dst.y as u16);
        if level_cell.blocked {
            std::u32::MAX
        } else if dst == target {
            1
        } else if level_cell.entities.iter().any(|e| e.blocked) {
            std::u32::MAX
        } else {
            1
        }
    }

    pub fn path(&self, src: &Position, dst: &Position) -> Option<(Vec<Position>,u32)> {
        let successors = |pos: &Position| {
            pos
                .neighbours()
                .into_iter()
                .map(|next| (next.clone(), self.move_cost(src, &next, dst)))
                .filter(|(_next,cost)| *cost != std::u32::MAX)
                .collect::<Vec<(Position,u32)>>()
        };

        let heuristic = |pos: &Position| pos.distance(&dst);

        let success = |pos: &Position| *pos == *dst;

        astar(&src, successors, heuristic, success)
    }
}