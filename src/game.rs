use crate::{prelude::*, render::GameRender};

pub struct GameState {
    geng: Geng,
    assets: Rc<Assets>,

    framebuffer_size: vec2<usize>,
    cursor: CursorState,
    render: GameRender,
    model: Model,
    drawing_sfx: geng::SoundEffect,
    volume: f32,
}

#[derive(Debug, Clone)]
pub struct CursorState {
    pub screen_pos: vec2<f64>,
    pub world_pos: Position,
}

impl GameState {
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        Self {
            geng: geng.clone(),
            assets: assets.clone(),

            framebuffer_size: vec2(1, 1),
            cursor: CursorState {
                screen_pos: vec2::ZERO,
                world_pos: vec2::ZERO,
            },
            render: GameRender::new(geng, assets),
            model: Model::new(assets.config.clone()),
            drawing_sfx: {
                let mut sfx = assets.sounds.drawing.play();
                sfx.set_volume(0.0);
                sfx
            },
            volume: 0.5,
        }
    }

    fn get_controls(&self) -> PlayerControls {
        let window = self.geng.window();

        let mut move_dir = vec2::<f32>::ZERO;
        if geng_utils::key::is_key_pressed(window, &self.assets.controls.left) {
            move_dir.x -= 1.0;
        }
        if geng_utils::key::is_key_pressed(window, &self.assets.controls.right) {
            move_dir.x += 1.0;
        }
        if geng_utils::key::is_key_pressed(window, &self.assets.controls.down) {
            move_dir.y -= 1.0;
        }
        if geng_utils::key::is_key_pressed(window, &self.assets.controls.up) {
            move_dir.y += 1.0;
        }

        let drawing = geng_utils::key::is_key_pressed(window, &self.assets.controls.draw)
            .then_some(self.cursor.world_pos);

        PlayerControls {
            move_dir: move_dir.as_r32(),
            drawing,
        }
    }

    fn play_sfx(&self, sfx: &geng::Sound) {
        let mut sfx = sfx.play();
        sfx.set_volume(self.volume);
    }
}

impl geng::State for GameState {
    fn update(&mut self, delta_time: f64) {
        let _delta_time = Time::new(delta_time as f32);
        self.cursor.world_pos = self
            .model
            .camera
            .screen_to_world(
                self.framebuffer_size.as_f32(),
                self.cursor.screen_pos.as_f32(),
            )
            .as_r32();
        self.model.cursor_pos = self.cursor.world_pos;
    }

    fn fixed_update(&mut self, delta_time: f64) {
        let delta_time = Time::new(delta_time as f32);

        let input = self.get_controls();
        self.model.update(input, delta_time);

        let mut drawing = false;
        let mut hit = false;
        let mut kill = false;
        for event in std::mem::take(&mut self.model.events) {
            match event {
                Event::Sound(sfx) => match sfx {
                    SoundEvent::Drawing => drawing = true,
                    SoundEvent::Hit => hit = true,
                    SoundEvent::Kill => kill = true,
                    SoundEvent::HitSelf => {
                        self.play_sfx(&self.assets.sounds.hit_self);
                    }
                    SoundEvent::Bounce => {
                        self.play_sfx(&self.assets.sounds.bounce);
                    }
                    SoundEvent::Expand => {
                        self.play_sfx(&self.assets.sounds.expand);
                    }
                },
            }
        }
        if kill {
            self.play_sfx(&self.assets.sounds.kill);
        } else if hit {
            self.play_sfx(&self.assets.sounds.hit);
        }
        self.drawing_sfx
            .set_volume(if drawing { 1.0 } else { 0.0 } * self.volume);
    }

    fn handle_event(&mut self, event: geng::Event) {
        if let geng::Event::CursorMove { position } = event {
            self.cursor.screen_pos = position;
        }
    }

    fn draw(&mut self, framebuffer: &mut geng::prelude::ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size();
        ugli::clear(
            framebuffer,
            Some(self.assets.palette.background),
            None,
            None,
        );

        self.render.draw_game(&self.model, framebuffer);
    }
}