use crate::prelude::*;

pub struct GameRender {
    geng: Geng,
    assets: Rc<Assets>,
}

impl GameRender {
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
        }
    }

    pub fn draw_game(&mut self, model: &Model, framebuffer: &mut ugli::Framebuffer) {
        if let Some(drawing) = &model.player.draw_action {
            // Drawing
            let points = drawing
                .points_smoothed
                .iter()
                .map(|pos| pos.as_f32())
                .collect();
            let chain = Chain::new(points);
            let chain = draw2d::Chain::new(chain, 0.1, Rgba::WHITE, 3);
            self.geng
                .draw2d()
                .draw2d(framebuffer, &model.camera, &chain);
        }

        // Objects
        for object in &model.objects {
            self.draw_collider(&object.collider, Rgba::RED, &model.camera, framebuffer);
        }

        // Enemies
        for enemy in &model.enemies {
            self.draw_collider(&enemy.collider, Rgba::CYAN, &model.camera, framebuffer);
            self.draw_health_bar(&enemy.collider, &enemy.health, &model.camera, framebuffer);
        }

        // Player
        self.draw_collider(
            &model.player.collider,
            Rgba::GREEN,
            &model.camera,
            framebuffer,
        );
        self.draw_health_bar(
            &model.player.collider,
            &model.player.health,
            &model.camera,
            framebuffer,
        );
    }

    pub fn draw_collider(
        &self,
        collider: &Collider,
        color: Rgba<f32>,
        camera: &Camera,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let transform = collider.transform_mat().as_f32();
        match &collider.shape {
            Shape::Circle { radius } => {
                self.geng.draw2d().draw2d_transformed(
                    framebuffer,
                    camera,
                    &draw2d::Ellipse::circle(vec2::ZERO, radius.as_f32(), color),
                    transform,
                );
            }
            &Shape::Rectangle { width, height } => {
                let quad = Aabb2::ZERO.extend_symmetric(vec2(width, height).as_f32() / 2.0);
                self.geng.draw2d().draw2d_transformed(
                    framebuffer,
                    camera,
                    &draw2d::Quad::new(quad, color),
                    transform,
                );
            }
        }
    }

    pub fn draw_health_bar(
        &self,
        collider: &Collider,
        health: &Health,
        camera: &Camera,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        if health.is_max() {
            return;
        }

        let aabb = collider.compute_aabb().map(Coord::as_f32);
        let health_bar = Aabb2::point(vec2(aabb.center().x, aabb.max.y + 0.2))
            .extend_symmetric(vec2(0.9, 0.2) / 2.0);

        // Outline
        self.geng
            .draw2d()
            .quad(framebuffer, camera, health_bar, Rgba::RED);
        let health_bar = health_bar.extend_uniform(-0.02);
        // Background
        self.geng
            .draw2d()
            .quad(framebuffer, camera, health_bar, Rgba::BLACK);
        // Fill
        let fill = health_bar.extend_symmetric(
            vec2(
                health_bar.width() * (health.get_ratio().as_f32() - 1.0),
                0.0,
            ) / 2.0,
        );
        self.geng
            .draw2d()
            .quad(framebuffer, camera, fill, Rgba::RED);
    }
}
