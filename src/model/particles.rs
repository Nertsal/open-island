use super::*;

#[derive(Debug, Clone)]
pub struct SpawnParticles {
    pub kind: ParticleKind,
    pub density: R32,
    pub distribution: ParticleDistribution,
    pub size: RangeInclusive<Coord>,
    pub velocity: vec2<Coord>,
    pub lifetime: RangeInclusive<Time>,
}

#[derive(Debug, Clone, Copy)]
pub enum ParticleKind {
    Draw,
    Drawing,
    WallBreakable,
    WallBlock,
    Bounce,
    Damage,
    Upgrade,
    HitSelf,
    Shield,
    Heal,
}

#[derive(Debug, Clone)]
pub enum ParticleDistribution {
    Circle { center: Position, radius: Coord },
    Drawing { points: Vec<Position>, width: Coord },
    Aabb(Aabb2<Coord>),
}

impl ParticleDistribution {
    pub fn sample(&self, rng: &mut impl Rng, density: R32) -> Vec<Position> {
        match self {
            &ParticleDistribution::Aabb(aabb) => {
                let amount = density * aabb.width() * aabb.height();
                let extra = if rng.gen_bool(amount.fract().as_f32().into()) {
                    1
                } else {
                    0
                };
                let amount = (amount.floor()).as_f32() as usize + extra;

                (0..amount)
                    .map(|_| {
                        vec2(
                            rng.gen_range(aabb.min.x..=aabb.max.x),
                            rng.gen_range(aabb.min.y..=aabb.max.y),
                        )
                    })
                    .collect()
            }
            &ParticleDistribution::Circle { center, radius } => {
                let amount = density * radius.sqr() * R32::PI;
                let extra = if rng.gen_bool(amount.fract().as_f32().into()) {
                    1
                } else {
                    0
                };
                let amount = (amount.floor()).as_f32() as usize + extra;

                (0..amount)
                    .map(|_| rng.gen_circle(center, radius))
                    .collect()
            }
            ParticleDistribution::Drawing { points, width } => points
                .windows(2)
                .flat_map(|segment| {
                    let &[a, b] = segment else { unreachable!() };
                    let n = (b - a).normalize_or_zero().rotate_90();

                    let amount = density * (b - a).len() * *width;
                    let extra = if rng.gen_bool(amount.fract().as_f32().into()) {
                        1
                    } else {
                        0
                    };
                    let amount = (amount.floor()).as_f32() as usize + extra;

                    let us: Vec<_> = rng
                        .sample_iter(rand::distributions::Uniform::new_inclusive(
                            -R32::ONE,
                            R32::ONE,
                        ))
                        .take(amount)
                        .collect();
                    let ts: Vec<_> = rng
                        .sample_iter(rand::distributions::Uniform::new_inclusive(
                            R32::ZERO,
                            R32::ONE,
                        ))
                        .zip(us)
                        .take(amount)
                        .collect();
                    ts.into_iter()
                        .map(move |(t, u)| a + (b - a) * t + n * *width * u)
                })
                .collect(),
        }
    }
}

impl Default for SpawnParticles {
    fn default() -> Self {
        Self {
            kind: ParticleKind::Draw,
            density: r32(5.0),
            distribution: ParticleDistribution::Circle {
                center: vec2::ZERO,
                radius: r32(0.5),
            },
            size: r32(0.05)..=r32(0.15),
            velocity: vec2::ZERO,
            lifetime: r32(0.5)..=r32(1.5),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Particle {
    pub kind: ParticleKind,
    pub collider: Collider,
    pub velocity: vec2<Coord>,
    pub lifetime: Bounded<Time>,
}
