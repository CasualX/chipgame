use super::*;

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
	Skating,
	/// Sliding on force floor without suction boots.
	Sliding,
	/// Walking on force floor with suction boots.
	Suction,
	/// Player drowned in water without flippers.
	Drowned,
	/// Player stepped in fire without fire boots.
	Burned,
	/// Player stepped on a bomb.
	Bombed,
	/// Player is out of time.
	OutOfTime,
	/// Player entity collided with a block.
	Collided,
	/// Player entity eaten by a creature.
	Eaten,
	/// Player entity does not exist.
	NotOkay,
	/// Player won the game.
	Win,
}

impl PlayerActivity {
	pub fn is_game_over(self) -> bool {
		matches!(self, PlayerActivity::Drowned | PlayerActivity::Burned | PlayerActivity::Bombed | PlayerActivity::OutOfTime | PlayerActivity::Collided | PlayerActivity::Eaten | PlayerActivity::NotOkay | PlayerActivity::Win)
	}
}

/// Player state.
#[derive(Clone, Default)]
pub struct PlayerState {
	pub ehandle: EntityHandle,

	/// Player input manager.
	pub inbuf: InputBuffer,

	/// Current player activity.
	pub activity: PlayerActivity,
	/// True if previous movement was involuntary.
	pub forced_move: bool,
	/// Total steps taken (for high score).
	pub steps: i32,
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

impl PlayerState {
	pub fn clear(&mut self) {
		self.inbuf = InputBuffer::default();
		self.activity = PlayerActivity::Walking;
		self.forced_move = false;
		self.steps = 0;
		self.chips = 0;
		self.keys = [0; 4];
		self.flippers = false;
		self.fire_boots = false;
		self.ice_skates = false;
		self.suction_boots = false;
		self.dev_wtw = false;
	}
}

pub(super) fn ps_update_inbuf(s: &mut GameState, input: &Input) {
	if !(s.input.a && s.input.b) && input.a && input.b {
		s.ps.dev_wtw = !s.ps.dev_wtw;
	}

	s.ps.inbuf.handle(Compass::Left,  input.left,  s.input.left);
	s.ps.inbuf.handle(Compass::Right, input.right, s.input.right);
	s.ps.inbuf.handle(Compass::Up,    input.up,    s.input.up);
	s.ps.inbuf.handle(Compass::Down,  input.down,  s.input.down);
}

pub(super) fn ps_activity(s: &mut GameState, activity: PlayerActivity) {
	if s.ps.activity != activity {
		s.ps.activity = activity;
		s.events.push(GameEvent::PlayerActivity { player: s.ps.ehandle });

		if activity.is_game_over() {
			s.ts = TimeState::Paused;
		}

		match activity {
			PlayerActivity::Drowned => {
				s.events.push(GameEvent::GameOver { player: s.ps.ehandle });
				s.events.push(GameEvent::SoundFx { sound: SoundFx::WaterSplash });
			}
			PlayerActivity::Burned => {
				s.events.push(GameEvent::GameOver { player: s.ps.ehandle });
				s.events.push(GameEvent::SoundFx { sound: SoundFx::FireWalking });
			}
			PlayerActivity::Bombed => {
				s.events.push(GameEvent::GameOver { player: s.ps.ehandle });
				s.events.push(GameEvent::SoundFx { sound: SoundFx::BombExplosion });
			}
			PlayerActivity::OutOfTime => {
				s.events.push(GameEvent::GameOver { player: s.ps.ehandle });
				s.events.push(GameEvent::SoundFx { sound: SoundFx::OutOfTime });
			}
			PlayerActivity::Collided => {
				s.events.push(GameEvent::GameOver { player: s.ps.ehandle });
				s.events.push(GameEvent::SoundFx { sound: SoundFx::GameOver });
			}
			PlayerActivity::Eaten => {
				s.events.push(GameEvent::GameOver { player: s.ps.ehandle });
				s.events.push(GameEvent::SoundFx { sound: SoundFx::GameOver });
			}
			PlayerActivity::Win => {
				s.events.push(GameEvent::GameWin { player: s.ps.ehandle });
				s.events.push(GameEvent::SoundFx { sound: SoundFx::GameWin });
			}
			_ => (),
		}
	}
}
