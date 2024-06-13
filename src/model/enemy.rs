use super::*;

#[derive(Debug, Clone)]
pub struct Enemy {
    pub health: Health,
    pub body: PhysicsBody,
    pub stats: EnemyConfig,
    pub ai: EnemyAI,
}

impl Enemy {
    pub fn new(config: EnemyConfig, position: Position) -> Self {
        Self {
            health: Bounded::new_max(config.health),
            body: PhysicsBody::new(position, config.shape),
            ai: config.ai.clone(),
            stats: config,
        }
    }

    pub fn move_rotation(&mut self) {
        self.body.angular_velocity = Angle::from_radians(
            self.body.velocity.len() * self.body.velocity.x.signum() / r32(2.0),
        );
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnemyAI {
    Idle,
    Bullet,
    Crawler,
    Shooter {
        preferred_distance: Coord,
        charge: Bounded<Time>,
        bullet: Box<EnemyConfig>,
    },
    Pacman {
        #[serde(default)]
        pacman: PacmanAI,
    },
    Helicopter {
        #[serde(default)]
        helicopter: HelicopterAI,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacmanAI {
    pub state: PacmanState,
    pub speed_power: Coord,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PacmanState {
    Normal {
        spawn_1up: Bounded<Time>,
        target: Option<Position>,
    },
    Power {
        timer: Bounded<Time>,
    },
}

impl Default for PacmanAI {
    fn default() -> Self {
        Self {
            state: PacmanState::Normal {
                spawn_1up: Bounded::new_max(r32(5.0)),
                target: None,
            },
            speed_power: r32(9.0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Pacman1Up {
    pub collider: Collider,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelicopterAI {
    pub oscilate: Bounded<Time>,
    pub state: HelicopterState,
    pub minigun_bullet: Box<EnemyConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HelicopterState {
    Idle,
    Moving(Position),
    Minigun {
        timer: Time,
        shot_delay: Bounded<Time>,
    },
    Minions {
        minions: Vec<EnemyConfig>,
        delay: Bounded<Time>,
    },
}

impl Default for HelicopterAI {
    fn default() -> Self {
        Self {
            oscilate: Bounded::new_zero(r32(7.0)),
            state: HelicopterState::Idle,
            minigun_bullet: Box::new(EnemyConfig {
                cost: None,
                score: None,
                health: r32(1.0),
                damage: r32(5.0),
                speed: r32(15.0),
                acceleration: r32(100.0),
                shape: Shape::circle(0.2),
                ai: EnemyAI::Bullet,
            }),
        }
    }
}
