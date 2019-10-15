use specs::{World, WorldExt, Join, Entity, Component, Read, ReadStorage, System, WriteStorage, VecStorage};
use hibitset::BitSetLike;
use std::time::Duration;
use std::collections::BTreeMap;
use std::cmp;
use super::level::{Level};
use crate::color::{Color};

pub struct Index<T> {
    blocked: BTreeMap<Position, T>,
}

impl <T> Index<T> {
    pub fn new() -> Self {
        Index {
            blocked: BTreeMap::new(),
        }
    }

    pub fn add(&mut self, pos: Position, e: T) {
        self.blocked.insert(pos, e);
    }

    pub fn remove(&mut self, pos: &Position) -> Option<T> {
        self.blocked.remove(pos)
    }

    pub fn get(&self, pos: &Position) -> Option<&T> {
        self.blocked.get(pos)
    }

    pub fn clear(&mut self) {
        self.blocked.clear();
    }

    pub fn is_blocked(&self, pos: &Position) -> bool {
        self.blocked.contains_key(pos)
    }

    pub fn move_to(&mut self, from: &Position, to: Position) {
        if let Some(old) = self.remove(from) {
            self.blocked.insert(to, old);
        }
    }
}

pub type EntityIndex = Index<Entity>;

#[derive(Debug, Default)]
pub struct PlayerController;

impl Component for PlayerController {
	type Storage = specs::NullStorage<Self>;
}

#[derive(Debug, Default)]
pub struct AiController;

impl Component for AiController {
	type Storage = specs::NullStorage<Self>;
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Attributes {
    pub name: String,
    pub blocks: bool,
    pub alive: bool,
    pub goodness: f32,
    pub lawfulness: f32,
    pub calmness: f32,
    pub thirst: f32,
    pub max_hp: i32,
    pub hp: i32,
    pub vision_radius: u16,
    pub faction: String,
}

impl Component for Attributes {
    type Storage = VecStorage<Self>;
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Liquid {
    pub potable: bool,
}

impl Component for Liquid {
    type Storage = VecStorage<Self>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct Fighter {
    pub defense: i32,
    pub attack: i32,
}

impl Component for Fighter {
    type Storage = VecStorage<Self>;
}

#[derive(Default)]
pub struct DeltaTime(Duration);

#[derive(Default, Debug, PartialEq, Clone)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Rect {
    pub fn new(left: i32, top: i32, width: i32, height: i32) -> Rect {
        Rect {
            x: left,
            y: top,
            width: width,
            height: height,
        }
    }

    pub fn new_sized(width: i32, height: i32) -> Rect {
        Self::new(0, 0, width, height)
    }

    pub fn left(&self) -> i32 {
        self.x
    }

    pub fn right(&self) -> i32 {
        self.x + self.width
    }

    pub fn top(&self) -> i32 {
        self.y
    }

    pub fn bottom(&self) -> i32 {
        self.y + self.height
    }

    pub fn area(&self) -> i32 {
        self.width * self.height
    }

    pub fn top_left(&self) -> Position {
        Position::new(self.x, self.y)
    }

    pub fn bottom_right(&self) -> Position {
        Position::new(self.x + self.width, self.y + self.height)
    }

    pub fn union(&self, other: &Rect) -> Rect {
        let left = cmp::min(self.x, other.x);
        let top = cmp::min(self.y, other.y);
        let right = cmp::max(self.x, other.x);
        let bottom = cmp::max(self.y, other.y);

        Rect {
            x: left,
            y: top,
            width: right - left,
            height: bottom - top,
        }
    }

    pub fn center(&self) -> Position {
        Position::new(
            self.x + self.width / 2,
            self.y + self.height / 2,
        )
    }

    pub fn inner(&self, horizontal_margin: i32, vertical_margin: i32) -> Rect {
        let horizontal_margin = cmp::min(horizontal_margin, self.width / 2);
        let vertical_margin = cmp::min(vertical_margin, self.height / 2);

        let left = self.x + horizontal_margin;
        let right = self.right() - horizontal_margin;
        let top = self.y + vertical_margin;
        let bottom = self.bottom() - vertical_margin;

        Rect {
            x: left,
            y: top,
            width: right - left,
            height: bottom - top,
        }
    }

    pub fn intersection(&self, other: &Rect) -> Rect {
        let left = cmp::max(self.x, other.x);
        let top = cmp::max(self.y, other.y);
        let right = cmp::min(self.right(), other.right());
        let bottom = cmp::min(self.bottom(), other.bottom());

        Rect {
            x: left,
            y: top,
            width: right - left,
            height: bottom - top,
        }
    }

    pub fn intersects(&self, other: &Rect) -> bool {
        other.left() < self.right()
        && other.right() > self.left()
        && other.top() < self.bottom()
        && other.bottom() > self.top()
    }
}

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Component for Position {
    type Storage = VecStorage<Self>;
}

impl Position {
    pub fn new(x: i32, y: i32) -> Position {
        Position {
            x: x,
            y: y,
        }
    }

    pub fn delta(&self, dx: i32, dy: i32) -> Position {
        Position {
            x: self.x + dx,
            y: self.y + dy,
        }
    }

    /// Calculate vector from this object to other object
    pub fn diff(&self, other: &Position) -> Distance {
        Distance {
            dx: other.x - self.x,
            dy: other.y - self.y,
        }
    }

    pub fn distance(&self, other: &Position) -> u32 {
        use pathfinding::prelude::absdiff;
        (absdiff(self.x, other.x) + absdiff(self.y, other.y)) as u32
    }

    pub fn neighbours(&self) -> Vec<Position> {
        vec![self.delta(-1,-1), self.delta(0,-1), self.delta(1,-1),
             self.delta(-1,0), self.delta(1,0),
             self.delta(-1,1), self.delta(0,1), self.delta(1,1)]
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Distance {
    pub dx: i32,
    pub dy: i32,
}

impl Distance {
    pub fn magnitude(&self) -> f32 {
        ((self.dx.pow(2) + self.dy.pow(2)) as f32).sqrt()
    }

    pub fn normalize(&self) -> Distance {
        let m = self.magnitude();
        Distance {
            dx: (self.dx as f32 / m).round() as i32,
            dy: (self.dy as f32 / m).round() as i32,
        }
    }
}

#[derive(Debug)]
pub struct Velocity {
    x: f32,
    y: f32,
}

impl Component for Velocity {
    type Storage = VecStorage<Self>;
}

pub struct UpdatePos;

impl<'a> System<'a> for UpdatePos {
    type SystemData = (WriteStorage<'a, Position>, ReadStorage<'a, Velocity>, Read<'a, DeltaTime>);

    fn run(&mut self, (mut positions, velocities, delta): Self::SystemData) {
        let _delta = delta.0;
        for (_vel, _pos) in (&velocities, &mut positions).join() {
            
        }
    }
}

pub struct Character {
	pub glyph: char,
    pub color: Color,
}

impl Component for Character {
	type Storage = VecStorage<Self>;
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum PlayerAlive {
    Alive,
    Dead,
}

impl Default for PlayerAlive {
    fn default() -> Self  {
        PlayerAlive::Alive
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Game {
    pub stop: bool,
    pub player_turn_ended: bool,
    pub player_alive: PlayerAlive,
}

impl Game {
    pub fn new() -> Self {
        Game::default()
    }

    pub fn end_turn(&self) -> Self {
        let mut g = self.clone();
        g.player_turn_ended = true;
        g
    }
}

pub struct Joiner<J, T, S, BS> where J: Join<Type=T, Value=S, Mask=BS>, BS: BitSetLike {
    join_tuple: J,
}

impl <J, T, S, BS> Joiner<J, T, S, BS> where J: Join<Type=T, Value=S, Mask=BS>, BS: BitSetLike {
    pub fn new(join_tuple: J) -> Self {
        Joiner {
            join_tuple: join_tuple,
        }
    }

    pub fn any<F>(self, mut f: F) -> bool where F: FnMut(T) -> bool {
        for t in self.join_tuple.join() {
            if f(t) {
                return true;
            }
        }

        false
    }

    pub fn all<F>(self, mut f: F) -> bool where F: FnMut(T) -> bool {
        for t in self.join_tuple.join() {
            if !f(t) {
                return false;
            }
        }

        true
    }
}

pub struct Collider<'a, T> {
    level_map: &'a Level,
    index: Index<T>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Occupier<T> where T: Clone {
    Empty,
    Wall,
    Entity(T)
}

pub trait Blockable {
    fn is_empty(&self) -> bool;
}

impl <T> Blockable for Occupier<T> where T: Clone {
    fn is_empty(&self) -> bool {
        match self {
            Occupier::Empty => true,
            _ => false,
        }
    }
}

impl <'a,T> Collider<'a,T> where T: Clone {
    pub fn new(level_map: &'a Level) -> Self {
        Collider {
            level_map: level_map,
            index: Index::new(),
        }
    }

    pub fn clear_index(&mut self) {
        self.index.clear();
    }

    pub fn index_mut<'b>(&'b mut self) -> &'b mut Index<T> {
        &mut self.index
    }

    pub fn get(&self, p: &Position) -> Occupier<T> {
        if self.level_map.get(p.x, p.y).blocked {
            return Occupier::Wall;
        }
        
        if let Some(entity) = self.index.get(&p) {
            return Occupier::Entity(entity.clone());
        }

        Occupier::Empty
    }
}

pub fn setup(world: &mut World) {
    world.register::<Position>();
    world.register::<Velocity>();
    world.register::<PlayerController>();
    world.register::<AiController>();
    world.register::<Attributes>();
    world.register::<Liquid>();
    world.register::<Fighter>();
    world.register::<Character>();
}