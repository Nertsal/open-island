use super::*;

impl Model {
    pub fn controls(&mut self, input: PlayerControls, delta_time: Time) {
        let can_expand = self.can_expand();
        let player = &mut self.player;

        // Invincibility
        let invincible = player.invincibility.is_above_min();
        player.invincibility.change(-delta_time);
        if invincible && !player.invincibility.is_above_min() {
            self.particles_queue.push(SpawnParticles {
                kind: ParticleKind::Shield,
                distribution: ParticleDistribution::Circle {
                    center: player.body.collider.position,
                    radius: r32(0.6),
                },
                ..default()
            });
        }

        // Movement
        if player.draw_action.is_some() {
            player.body.velocity = vec2::ZERO;
        } else {
            let move_dir = input.move_dir.clamp_len(..=Coord::ONE);
            let target_velocity = move_dir * player.stats.speed;
            player.body.velocity += (target_velocity - player.body.velocity)
                .clamp_len(..=player.stats.acceleration * delta_time);
        }
        player.body.collider.position += player.body.velocity * delta_time;
        // player.body.collider.rotation += player.body.angular_velocity * delta_time;
        player.body.collider.rotation = (self.cursor_pos - player.body.collider.position).arg()
            + Angle::from_degrees(30.0).map(r32);

        match input.drawing {
            Some(position) => {
                // Drawing
                if player.draw_action.is_none() {
                    player.draw_action = Some(Drawing {
                        points_raw: vec![DrawPoint {
                            position: player.body.collider.position,
                            time: self.real_time,
                        }],
                        points_smoothed: Vec::new(),
                    });
                }

                let mut point = DrawPoint {
                    position,
                    time: self.real_time,
                };

                let drawing = player.draw_action.as_mut().unwrap();
                let remaining = player.stats.dash.max_distance - drawing.length();

                let inside = self
                    .rooms
                    .iter()
                    .any(|(_, room)| room.area.contains(point.position));
                let direction = self
                    .rooms
                    .iter()
                    .find(|(_, room)| room.area.contains(player.body.collider.position))
                    .map(|(_, room)| (room, room.closest_wall(point.position)));
                let can_expand = can_expand
                    && direction.map_or(false, |(room, (_, direction))| {
                        room.expanded_direction.is_none()
                            && room
                                .unlocked_after
                                .map_or(true, |(_, dir)| dir != direction)
                    });

                if remaining > Coord::ZERO && (inside || can_expand) {
                    // Add a point
                    let last = drawing
                        .points_raw
                        .last()
                        .expect("drawing has to have a starting point");
                    // Clamp max dash distance
                    point.position =
                        last.position + (point.position - last.position).clamp_len(..=remaining);
                    drawing.points_raw.push(point);

                    // Update smooth
                    let points: Vec<_> = drawing
                        .points_raw
                        .iter()
                        .map(|point| point.position.as_f32())
                        .dedup_by(|a, b| (*a - *b).len_sqr() < 0.01)
                        .collect();
                    let chain = if points.len() < 3 {
                        Chain::new(points)
                    } else {
                        CardinalSpline::new(points, 0.5).chain(3)
                    };
                    drawing.points_smoothed =
                        chain.vertices.into_iter().map(|pos| pos.as_r32()).collect();
                }

                self.particles_queue.push(SpawnParticles {
                    density: r32(0.5),
                    kind: ParticleKind::Draw,
                    distribution: ParticleDistribution::Drawing {
                        points: drawing.points_smoothed.clone(),
                        width: r32(0.2),
                    },
                    ..default()
                });
            }
            None => self.stop_drawing(),
        }
    }

    pub fn stop_drawing(&mut self) {
        let Some(drawing) = self.player.draw_action.take() else {
            return;
        };

        self.player_dash(drawing);
    }

    pub fn player_dash(&mut self, drawing: Drawing) {
        if drawing.points_smoothed.len() < 2 {
            return;
        }

        let can_expand = self.can_expand();
        let expand_room = can_expand
            .then(|| {
                self.rooms
                    .iter()
                    .find(|(_, room)| room.area.contains(self.player.body.collider.position))
                    .map(|(idx, _)| idx)
            })
            .flatten();

        let &last = drawing.points_smoothed.last().unwrap();
        let &prelast = drawing
            .points_smoothed
            .get(drawing.points_smoothed.len() - 2)
            .unwrap();

        self.player.body.collider.position = last;
        self.player.body.velocity =
            (last - prelast).normalize_or_zero() * self.config.player.dash.speed;
        self.player.invincibility.set_ratio(Time::ONE);

        self.damage_around(
            drawing,
            self.player.stats.dash.width,
            self.player.stats.dash.damage,
        );

        if let Some(room) = expand_room {
            if !self
                .rooms
                .iter()
                .any(|(_, room)| room.area.contains(self.player.body.collider.position))
            {
                self.unlock_room(room, self.player.body.collider.position);
            }
        }
    }
}
