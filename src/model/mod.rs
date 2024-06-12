mod collider;
mod logic;
mod particles;

pub use self::{collider::*, particles::*};

use crate::prelude::*;

pub type Camera = Camera2d;
pub type Coord = R32;
pub type Position = vec2<Coord>;
pub type Time = R32;
pub type Hp = R32;
pub type Health = Bounded<Hp>;

pub struct Model {
    pub config: Config,
    pub camera: Camera,
    pub real_time: Time,
    pub game_time: Time,
    pub cursor_pos: Position,

    pub player: Player,
    pub rooms: Arena<Room>,
    pub room_colliders: Vec<(Index, Direction, Collider)>,
    pub objects: Vec<Object>,
    pub enemies: Vec<Enemy>,
    pub particles: Arena<Particle>,

    pub particles_queue: Vec<SpawnParticles>,
    pub events: Vec<Event>,
}

#[derive(Debug)]
pub enum Event {
    Sound(SoundEvent),
}

#[derive(Debug)]
pub enum SoundEvent {
    Drawing,
    Hit,
    Kill,
    HitSelf,
    Bounce,
    Expand,
}

#[derive(Debug, Clone)]
pub struct Room {
    pub area: Aabb2<Coord>,
    /// Index of the room the player unlocked this room from.
    pub unlocked_after: Option<(Index, Direction)>,
    pub expanded_direction: Option<Direction>,
}

impl Room {
    pub fn closest_wall(&self, pos: Position) -> (Coord, Direction) {
        let mut dist = r32(9999999999.0);
        let mut closest = Direction::Left;
        let left = self.area.min.x - pos.x;
        if left > Coord::ZERO && left < dist {
            dist = left;
            closest = Direction::Left;
        }
        let right = pos.x - self.area.max.x;
        if right > Coord::ZERO && right < dist {
            dist = right;
            closest = Direction::Right;
        }
        let bottom = self.area.min.y - pos.y;
        if bottom > Coord::ZERO && bottom < dist {
            dist = bottom;
            closest = Direction::Down;
        }
        let top = pos.y - self.area.max.y;
        if top > Coord::ZERO && top < dist {
            dist = top;
            closest = Direction::Up;
        }
        (dist, closest)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
    Down,
    Up,
}

impl Direction {
    pub fn opposite(&self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Down => Self::Up,
            Self::Up => Self::Down,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Object {
    pub collider: Collider,
}

#[derive(Debug, Clone)]
pub struct PhysicsBody {
    pub collider: Collider,
    pub velocity: vec2<Coord>,
    pub angular_velocity: Angle<R32>,
}

impl PhysicsBody {
    pub fn new(position: Position, shape: Shape) -> Self {
        Self {
            collider: Collider::new(position, shape),
            velocity: vec2::ZERO,
            angular_velocity: Angle::ZERO,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Enemy {
    pub health: Health,
    pub body: PhysicsBody,
    pub stats: EnemyConfig,
    pub ai: EnemyAI,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnemyAI {
    Idle,
    Crawler,
}

#[derive(Debug, Clone)]
pub struct Player {
    pub health: Health,
    pub body: PhysicsBody,
    pub stats: PlayerConfig,
    pub draw_action: Option<Drawing>,
}

#[derive(Debug, Clone)]
pub struct Drawing {
    pub points_raw: Vec<DrawPoint>,
    pub points_smoothed: Vec<Position>,
}

impl Drawing {
    pub fn length(&self) -> Coord {
        self.points_raw
            .windows(2)
            .map(|segment| (segment[1].position - segment[0].position).len())
            .fold(Coord::ZERO, Coord::add)
    }
}

#[derive(Debug, Clone)]
pub struct DrawPoint {
    pub position: Position,
    pub time: Time,
}

#[derive(Debug, Clone)]
pub struct PlayerControls {
    pub move_dir: vec2<Coord>,
    pub drawing: Option<vec2<Coord>>,
}

impl Model {
    pub fn new(config: Config) -> Self {
        let mut rooms = Arena::new();
        rooms.insert(Room {
            area: Aabb2::ZERO.extend_symmetric(config.starting_area / r32(2.0)),
            unlocked_after: None,
            expanded_direction: None,
        });

        let mut model = Self {
            camera: Camera {
                center: vec2::ZERO,
                rotation: Angle::ZERO,
                fov: 30.0,
            },
            real_time: Time::ZERO,
            game_time: Time::ZERO,
            cursor_pos: vec2::ZERO,

            player: Player {
                health: Health::new_max(config.player.health),
                body: PhysicsBody::new(vec2::ZERO, Shape::circle(0.5)),
                stats: config.player.clone(),
                draw_action: None,
            },
            rooms,
            room_colliders: Vec::new(),
            objects: vec![],
            enemies: vec![],
            particles: Arena::new(),

            particles_queue: Vec::new(),
            events: Vec::new(),

            config,
        };
        model.update_room_colliders();
        model
    }
}