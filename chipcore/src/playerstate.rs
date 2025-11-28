use super::*;

/// Multiplayer player index.
pub type PlayerIndex = ();

/// Player activity.
#[derive(Copy, Clone, Default, Debug, Eq, PartialEq)]
pub enum PlayerActivity {
	#[default]
	/// Walking around.
	Walking,
	/// Pushing a block.
	Pushing,
	/// Swimming in water with flippers.
	Swimming,
	/// Sliding on ice without ice skates.
	IceSliding,
	/// Skating on ice with ice skates.
	IceSkating,
	/// Sliding on force floor without suction boots.
	ForceSliding,
	/// Walking on force floor with suction boots.
	ForceWalking,
}

/// Player state.
#[derive(Clone, Default)]
pub struct PlayerState {
	pub master: EntityHandle,

	/// Player input manager.
	pub inbuf: InputBuffer,

	/// Current player activity.
	pub activity: PlayerActivity,
	/// Last step direction for block slapping.
	pub last_step_dir: Option<Compass>,
	/// Total steps taken (for high score).
	pub steps: i32,
	/// Total bonks into walls.
	pub bonks: i32,
	/// Number of attempts (for high score).
	pub attempts: i32,
	/// Total chips collected.
	pub chips: i32,
	/// Keys collected.
	pub keys: [u8; 4],

	pub flippers: bool,
	pub fire_boots: bool,
	pub ice_skates: bool,
	pub suction_boots: bool,
	pub dev_wtw: bool,
}

pub(super) fn ps_input(s: &mut GameState, input: &Input) {
	s.inputs.push(input.encode());
	s.ps.inbuf.handle(Compass::Left,  input.left,  s.input.left);
	s.ps.inbuf.handle(Compass::Right, input.right, s.input.right);
	s.ps.inbuf.handle(Compass::Up,    input.up,    s.input.up);
	s.ps.inbuf.handle(Compass::Down,  input.down,  s.input.down);
}

pub(super) fn ps_activity(s: &mut GameState, ehandle: EntityHandle, activity: PlayerActivity) {
	if s.ps.activity != activity {
		s.ps.activity = activity;
		s.events.fire(GameEvent::PlayerActivity { entity: ehandle });
	}
}

/// Returns if the player entity is at the given position.
fn ps_check_pos(s: &GameState, pos: Vec2i) -> bool {
	s.ents.get(s.ps.master).map(|e| e.pos) == Some(pos)
}

/// Returns if the player entity is at the given position.
pub fn ps_check_new_pos(s: &GameState, pos: Vec2i) -> bool {
	if let Some(pl) = s.ents.get(s.ps.master) {
		if pl.pos == pos && pl.flags & EF_NEW_POS != 0 {
			return true;
		}
	}
	return false;
}

pub fn ps_attack(s: &mut GameState, entity: EntityHandle, reason: GameOverReason) {
	// Play game over jingle
	let sound = match reason {
		GameOverReason::LevelComplete => Some(SoundFx::GameWin),
		GameOverReason::Drowned => Some(SoundFx::WaterSplash),
		GameOverReason::Burned => Some(SoundFx::FireWalking),
		GameOverReason::Bombed => None, // Already fired by the Bomb entity!
		GameOverReason::Collided => Some(SoundFx::GameOver),
		GameOverReason::Eaten => Some(SoundFx::GameOver),
		GameOverReason::TimeOut => Some(SoundFx::GameOver),
		GameOverReason::NotOkay => None,
	};
	if let Some(sound) = sound {
		s.events.fire(GameEvent::SoundFx { sound });
	}

	s.events.fire(GameEvent::PlayerGameOver { entity, reason });
	s.game_over(reason);
}

/// Attacks the given position, harming the player if they are there.
pub fn ps_attack_pos(s: &mut GameState, pos: Vec2i, reason: GameOverReason) {
	if ps_check_pos(s, pos) {
		ps_attack(s, s.ps.master, reason);
	}
}

/// Triggers game over for the player.
pub fn ps_game_over(s: &mut GameState, reason: GameOverReason) {
	ps_attack(s, s.ps.master, reason);
}

/// Returns the nearest player entity to the given position.
pub fn ps_nearest_ent(s: &GameState, _pos: Vec2i) -> Option<&Entity> {
	s.ents.get(s.ps.master)
}
