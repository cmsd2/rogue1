use std::cmp;
use rand::{self, Rng};
use specs::{Entity, World, Builder};
use specs::world::WorldExt;
use crate::game::factions;
use super::level::{self, Tile, TileType, Level};
use crate::color::{Hue, Palette};
use super::ecs::{AiController, Attributes, Character, Position, Rect, EntityIndex, Fighter, PlayerController};
use super::fov::Fov;

const ROOM_MAX_SIZE: i32 = 10;
const ROOM_MIN_SIZE: i32 = 6;
const MAX_ROOMS: i32 = 30;
const MAX_ROOM_MONSTERS: i32 = 3;

pub fn fill_rect<F>(room: &Rect, map: &mut Level, f: F) where F: Fn(&mut Tile) {
    for x in (room.left())..(room.right()) {
        for y in (room.top())..(room.bottom()) {
            let tile = map.get_mut(x, y);
            f(tile);
        }
    }
}

pub fn create_room(room: &Rect, map: &mut Level) {
    fill_rect(&room, map, |tile| {
        match tile.cell_type {
            TileType::Void => {
                *tile = Tile::wall();
            },
            _ => {}
        }
    });
    fill_rect(&room.inner(1, 1), map, |tile| {
        *tile = Tile::floor();
    });
}

pub fn create_h_tunnel(x1: i32, x2: i32, y: i32, map: &mut Level) {
    let left = cmp::min(x1, x2);
    let right = cmp::max(x1, x2);
    let top = y - 1;
    let bottom = y + 1;
    let rect = Rect::new(left, top, right - left + 1, bottom - top + 1);
    create_room(&rect, map);
}

pub fn create_v_tunnel(y1: i32, y2: i32, x: i32, map: &mut Level) {
    let top = cmp::min(y1, y2);
    let bottom = cmp::max(y1, y2);
    let left = x - 1;
    let right = x + 1;
    let rect = Rect::new(left, top, right - left + 1, bottom - top + 1);
    create_room(&rect, map);
}

pub fn make_map(palette: &Palette, level_map: &mut Level, world: &mut World) -> Vec<Entity> {
    let mut entities = vec![];

    let area = level_map.area().clone();
    fill_rect(&area, level_map, |tile| {
        *tile = Tile::default();
    });

    let mut rooms: Vec<Rect> = vec![];
    let mut starting_position = Position::new(0, 0);

    for _ in 0..MAX_ROOMS {
        // random width and height
        let w = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);
        let h = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);
        // random position without going out of the boundaries of the map
        let x = rand::thread_rng().gen_range(level_map.area().x, level_map.area().right() - w);
        let y = rand::thread_rng().gen_range(level_map.area().y, level_map.area().bottom() - h);

        let new_room = Rect::new(x, y, w, h);

        // run through the other rooms and see if they intersect with this one
        let failed = rooms
            .iter()
            .any(|other_room| new_room.intersects(other_room));

        if !failed {
            // this means there are no intersections, so this room is valid

            // "paint" it to the map's tiles
            create_room(&new_room, level_map);

            // center coordinates of the new room, will be useful later
            let new_center = new_room.center();

            if rooms.is_empty() {
                // this is the first room, where the player starts at
                starting_position = new_center;
            } else {
                // all rooms after the first:
                // connect it to the previous room with a tunnel

                // center coordinates of the previous room
                let prev_center = rooms[rooms.len() - 1].center();

                // draw a coin (random bool value -- either true or false)
                if rand::random() {
                    // first move horizontally, then vertically
                    create_h_tunnel(prev_center.x, new_center.x, prev_center.y, level_map);
                    create_v_tunnel(prev_center.y, new_center.y, new_center.x, level_map);
                } else {
                    // first move vertically, then horizontally
                    create_v_tunnel(prev_center.y, new_center.y, prev_center.x, level_map);
                    create_h_tunnel(prev_center.x, new_center.x, new_center.y, level_map);
                }
            }

            place_objects(palette, new_room.clone(), level_map, world, &mut entities);

            rooms.push(new_room);
        }
    }

    //carve_walls(level_map);

    level_map.start = starting_position;

    entities
}

fn get_cell_type<'a>(level_map: &'a Level, x: i32, y: i32) -> Option<&'a TileType> {
    let area = level_map.area();
    if x < area.left() || x >= area.right() || y < area.top() || y >= area.bottom() {
        None
    } else {
        Some(&level_map.get(x, y).cell_type)
    }
}

fn is_wall(level_map: &Level, x: i32, y: i32) -> bool {
    match get_cell_type(level_map, x, y) {
        Some(TileType::Wall) => true,
        _ => false,
    }
}

pub fn carve_walls(level_map: &mut Level) {
    let area = level_map.area().to_owned();
    for i in area.left()..area.right() {
        for j in area.top()..area.bottom() {
            if is_wall(level_map, i as i32, j as i32) {
                let above = is_wall(level_map, i as i32, j as i32 - 1);
                let below = is_wall(level_map, i as i32, j as i32 + 1);
                let left = is_wall(level_map, i as i32 - 1, j as i32);
                let right = is_wall(level_map, i as i32 + 1, j as i32);
                let above_right = is_wall(level_map, i as i32 + 1, j as i32 - 1);
                let below_right = is_wall(level_map, i as i32 + 1, j as i32 + 1);
                let above_left = is_wall(level_map, i as i32 - 1, j as i32 - 1);
                let below_left = is_wall(level_map, i as i32 - 1, j as i32 + 1);

                let wall_character = match (left, right, above, below, above_right, below_right, above_left, below_left) {
                    (true, true, true, true, true, true, true, true) => ' ',
                    (true, true, true, true, true, true, true, false) => level::LINE_LIGHT_DOWN_AND_LEFT,
                    (true, true, true, true, true, true, false, true) => level::LINE_LIGHT_UP_AND_LEFT,
                    (true, true, true, true, true, true, false, false) => level::LINE_LIGHT_VERTICAL_AND_LEFT,
                    (true, true, true, true, true, false, true, true) => level::LINE_LIGHT_DOWN_AND_RIGHT,
                    (true, true, true, true, true, false, true, false) => level::LINE_LIGHT_HORIZONTAL_AND_DOWN,
                    (true, true, true, true, false, true, true, true) => level::LINE_LIGHT_UP_AND_RIGHT,
                    (true, true, true, true, false, true, false, true) => level::LINE_LIGHT_VERTICAL_AND_LEFT,
                    (true, true, true, true, false, false, true, true) => level::LINE_LIGHT_VERTICAL_AND_RIGHT,
                    (true, true, true, true, _, _, _, _) => level::LINE_LIGHT_VERTICAL_AND_HORIZONTAL,
                    
                    (true, true, true, false, true, _, true, _) => level::LINE_LIGHT_HORIZONTAL,
                    (true, true, true, false, _, _, _, _) => level::LINE_LIGHT_HORIZONTAL_AND_UP,
                    (true, true, false, true, _, true, _, true) => level::LINE_LIGHT_HORIZONTAL,
                    (true, true, false, true, _, _, _, _) => level::LINE_LIGHT_HORIZONTAL_AND_DOWN,
                    (true, true, false, false, _, _, _, _) => level::LINE_LIGHT_HORIZONTAL,
                    (true, false, true, true, _, _, true, true) => level::LINE_LIGHT_VERTICAL,
                    (true, false, true, true, _, _, _, _) => level::LINE_LIGHT_VERTICAL_AND_LEFT,
                    (true, false, true, false, _, _, _, _) => level::LINE_LIGHT_UP_AND_LEFT,
                    (true, false, false, true, _, _, _, _) => level::LINE_LIGHT_DOWN_AND_LEFT,
                    (true, false, false, false, _, _, _, _) => level::LINE_LIGHT_HORIZONTAL,
                    (false, true, true, true, true, true, _, _) => level::LINE_LIGHT_VERTICAL,
                    (false, true, true, true, _, _, _, _) => level::LINE_LIGHT_VERTICAL_AND_RIGHT,
                    (false, true, true, false, _, _, _, _) => level::LINE_LIGHT_UP_AND_RIGHT,
                    (false, true, false, true, _, _, _, _) => level::LINE_LIGHT_DOWN_AND_RIGHT,
                    (false, true, false, false, _, _, _, _) => level::LINE_LIGHT_HORIZONTAL,
                    (false, false, true, true, _, _, _, _) => level::LINE_LIGHT_VERTICAL,
                    (false, false, true, false, _, _, _, _) => level::LINE_LIGHT_VERTICAL,
                    (false, false, false, true, _, _, _, _) => level::LINE_LIGHT_VERTICAL,
                    (false, false, false, false, _, _, _, _) => level::LINE_LIGHT_BOX,
                };

                let tile = level_map.get_mut(i, j);
                tile.cell_type = TileType::Wall;
                tile.glyph = wall_character;
            }
        }
    }
}

fn place_objects(palette: &Palette, room: Rect, level_map: &mut Level, world: &mut World, entities: &mut Vec<Entity>) {
    // choose random number of monsters
    let num_monsters = rand::thread_rng().gen_range(0, MAX_ROOM_MONSTERS + 1);
    let mut index = EntityIndex::new();

    for _ in 0..num_monsters {
        // choose random spot for this monster
        let x = rand::thread_rng().gen_range(room.x + 1, room.right());
        let y = rand::thread_rng().gen_range(room.y + 1, room.bottom());

        if !index.is_blocked(&Position { x: x as i32, y: y as i32 }) {

            let (c,e,col) = if rand::random::<f32>() < 0.8 {  // 80% chance of getting an orc
                // create an orc
                let col = palette.color(Hue::Green, 128);
                let e = world.create_entity()
                    .with(Position { x: x as i32, y: y as i32 })
                    .with(Character { glyph: 'o', color: col.clone() })
                    .with(Attributes {
                        name: "orc".to_owned(),
                        blocks: true,
                        alive: true,
                        max_hp: 10,
                        hp: 10,
                        faction: factions::MONSTER.into(),
                        ..Default::default()
                    })
                    .with(AiController)
                    .with(Fighter {
                        defense: 0,
                        attack: 3,
                    })
                    .build();
                ('o', e, col)
            } else {
                let col = palette.color(Hue::Green, 255);
                let e = world.create_entity()
                    .with(Position { x: x as i32, y: y as i32 })
                    .with(Character { glyph: 'T', color: col.clone() })
                    .with(Attributes {
                        name: "troll".to_owned(),
                        blocks: true,
                        alive: true,
                        max_hp: 16,
                        hp: 16,
                        faction: factions::MONSTER.into(),
                        ..Default::default()
                    })
                    .with(AiController)
                    .with(Fighter {
                        defense: 1,
                        attack: 4,
                    })
                    .build();
                ('T', e, col)
            };

            entities.push(e);
            level_map.get_mut(x, y).entities.push(level::Entity {
                character: c,
                blocked: true,
                id: e,
                color: col,
            });
            index.add(Position { x: x as i32, y: y as i32 }, e);
        }
    }
}

pub fn create_player(palette: &Palette, level: &mut Level, fov: &mut Fov, world: &mut World) -> Entity {
    let vision_radius = 20;
    let start = level.start();

    fov.compute(&start, vision_radius);

    let col = palette.color(Hue::White, 255);
    let e = world
        .create_entity()
        .with(start.clone())
        .with(Character { glyph: '@', color: col.clone() })
        .with(PlayerController {})
        .with(Attributes {
            name: "player".to_owned(),
            blocks: true,
            alive: true,
            max_hp: 30,
            hp: 30,
            vision_radius: vision_radius,
            faction: factions::PLAYER.into(),
            ..Default::default()
        })
        .with(Fighter {
            defense: 2,
            attack: 5,
        })
        .build();
    
    level.get_mut(start.x, start.y).entities.push(level::Entity {
        character: '@',
        blocked: true,
        id: e,
        color: col,
    });

    e
}