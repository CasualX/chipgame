use super::*;

#[derive(Default)]
pub struct FxState {
	pub gs: chipcore::GameState,
	pub camera: PlayCamera,
	pub render: render::RenderState,
	pub objlookup: HashMap<chipcore::EntityHandle, render::ObjectHandle>,
	pub firesprites: HashMap<Vec2i, render::ObjectHandle>,
	pub togglewalls: HashMap<Vec2i, render::ObjectHandle>,
	pub inviswalls: HashMap<Vec2i, render::ObjectHandle>,
	pub level_number: i32,
	pub next_level_load: f32,
	pub game_realtime: f32,
	pub game_win: bool,
	pub hud_enabled: bool,
	pub darken: bool,
	pub darken_time: f32,
	pub events: Vec<FxEvent>,
}

impl FxState {
	pub fn parse_level(&mut self, level_number: i32, level_dto: &chipty::LevelDto) {
		self.gs.parse(level_dto, chipcore::RngSeed::System);

		// Reset the camera, adjusted when a player entity is created
		self.camera = PlayCamera::default();

		self.render.clear();
		self.objlookup.clear();
		self.firesprites.clear();
		self.togglewalls.clear();
		self.inviswalls.clear();

		self.render.field.width = self.gs.field.width;
		self.render.field.height = self.gs.field.height;
		self.render.field.terrain.extend_from_slice(&self.gs.field.terrain);
		for y in 0..self.gs.field.height {
			for x in 0..self.gs.field.width {
				let index = (y * self.gs.field.width + x) as usize;
				let terrain = self.gs.field.terrain[index];
				match terrain {
					chipty::Terrain::Fire => handlers::create_fire(self, Vec2::new(x, y)),
					chipty::Terrain::ToggleFloor => handlers::create_toggle_wall(self, Vec2(x, y), false),
					chipty::Terrain::ToggleWall => handlers::create_toggle_wall(self, Vec2(x, y), true),
					chipty::Terrain::HiddenWall | chipty::Terrain::InvisibleWall => handlers::create_invis_wall(self, Vec2(x, y)),
					_ => {}
				}
			}
		}

		self.level_number = level_number;
		self.next_level_load = 0.0;
		self.game_realtime = 0.0;
		self.game_win = false;
		self.hud_enabled = true;
		self.darken = true;
		self.darken_time = -1.0;

		self.events.clear();
		self.sync();
	}
	pub fn scout(&mut self) {
		self.gs.ts = chipcore::TimeState::Paused;
		self.events.push(FxEvent::Scout);
		self.hud_enabled = true;
	}
	pub fn pause(&mut self) {
		self.gs.ts = chipcore::TimeState::Paused;
		self.events.push(FxEvent::Pause);
		self.hud_enabled = false;
	}
	pub fn unpause(&mut self) {
		if matches!(self.gs.ts, chipcore::TimeState::Paused) {
			self.gs.ts = chipcore::TimeState::Running;
			self.events.push(FxEvent::Unpause);
			self.hud_enabled = true;

			// Center camera on player again
			self.camera.move_teleport = true;
		}
	}
	pub fn think(&mut self, input: &Input) {
		if !self.gs.ps.activity.is_game_over() {
			if input.start.is_pressed() {
				self.pause();
			}
			else if input.select.is_pressed() {
				self.scout();
			}
		}

		self.gs.tick(&chipcore::Input {
			a: input.a.is_held(),
			b: input.b.is_held(),
			up: input.up.is_held(),
			down: input.down.is_held(),
			left: input.left.is_held(),
			right: input.right.is_held(),
			start: input.start.is_held(),
			select: input.select.is_held(),
		});
		self.sync();

		if self.next_level_load != 0.0 && self.render.time > self.next_level_load {
			self.next_level_load = 0.0;
			self.events.push(if self.game_win { FxEvent::GameWin } else { FxEvent::GameOver });
		}

		// Update invisible walls based on player proximity
		if let Some(player) = self.gs.ents.get(self.gs.ps.ehandle) {
			for (&pos, &obj_handle) in &self.inviswalls {
				let Some(obj) = self.render.objects.get_mut(obj_handle) else { continue };
				if player.pos.distance_hat(pos) <= 2 {
					if obj.anim.anims.is_empty() && obj.data.alpha < 1.0 {
						obj.anim.anims.push(render::AnimState::FadeIn(render::FadeIn { atime: 0.0 }));
					}
				}
				else {
					if obj.anim.anims.is_empty() && obj.data.alpha > 0.0 {
						obj.anim.anims.push(render::AnimState::FadeOut(render::FadeOut { atime: 0.0 }));
					}
				}
			}
		}
	}
	/// Process game events and update FX state accordingly.
	pub fn sync(&mut self) {
		for ev in &self.gs.events.take() {
			eprintln!("GameEvent: {:?}", ev);
			match ev {
				&chipcore::GameEvent::EntityCreated { entity, kind } => handlers::entity_created(self, entity, kind),
				&chipcore::GameEvent::EntityRemoved { entity, kind } => handlers::entity_removed(self, entity, kind),
				&chipcore::GameEvent::EntityStep { entity } => handlers::entity_step(self, entity),
				&chipcore::GameEvent::EntityTurn { entity } => handlers::entity_face_dir(self, entity),
				&chipcore::GameEvent::EntityHidden { entity, hidden } => handlers::entity_hidden(self, entity, hidden),
				&chipcore::GameEvent::EntityTeleport { entity } => handlers::entity_teleport(self, entity),
				&chipcore::GameEvent::PlayerActivity { player } => handlers::player_activity(self, player),
				&chipcore::GameEvent::LockOpened { pos, key } => handlers::lock_opened(self, pos, key),
				&chipcore::GameEvent::FireHidden { pos, hidden } => handlers::fire_hidden(self, pos, hidden),
				&chipcore::GameEvent::TerrainUpdated { pos, old, new } => handlers::terrain_updated(self, pos, old, new),
				&chipcore::GameEvent::GameOver { player } => handlers::game_over(self, player),
				&chipcore::GameEvent::SoundFx { sound } => self.events.push(FxEvent::PlaySound { sound }),
				&chipcore::GameEvent::BombExplode { pos } => handlers::effect(self, pos, render::EffectType::Sparkles),
				&chipcore::GameEvent::WaterSplash { pos } => handlers::effect(self, pos, render::EffectType::Splash),
				_ => {}
			}
		}
	}
	pub fn follow_player(&mut self) {
		if matches!(self.gs.ts, chipcore::TimeState::Paused) {
			return;
		}
		if let Some(obj) = self.objlookup.get(&self.gs.ps.ehandle).and_then(|h| self.render.objects.get(*h)) {
			self.camera.set_target(obj.data.pos + Vec3(16.0, 16.0, 0.0));
		}
	}
	pub fn scout_dir(&mut self, dir: chipty::Compass, speed: f32) {
		self.camera.set_target(self.camera.target + dir.to_vec().vec3(0).cast::<f32>() * speed);
	}
	pub fn draw(&mut self, g: &mut shade::Graphics, resx: &Resources) {
		if self.gs.time != 0 {
			self.camera.animate_blend();
		}
		if !matches!(self.gs.ts, chipcore::TimeState::Paused) {
			self.camera.animate_move(self.render.time);
		}
		self.camera.animate_position();
		self.render.update();

		let camera = self.camera.setup(resx.viewport.size());
		self.render.draw(g, resx, &camera);

		if self.hud_enabled {
			self.render_ui(g, resx);
		}
	}
}
