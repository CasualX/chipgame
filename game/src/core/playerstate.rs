use super::*;

/// Player activity.
#[derive(Copy, Clone, Default, Debug, Eq, PartialEq)]
pub enum PlayerActivity {
	#[default]
	Walk,
	Push,
	Swim,
	Drown,
	Burn,
	Skate,
	Slide,
	Suction,
	Death,
	Win,
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
		self.activity = PlayerActivity::Walk;
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
		s.events.push(GameEvent::PlayerAction { player: s.ps.ehandle });
		if matches!(activity, PlayerActivity::Win) {
			s.ts = TimeState::Stopped;
			s.events.push(GameEvent::GameWin { player: s.ps.ehandle });
		}
		if matches!(activity, PlayerActivity::Burn | PlayerActivity::Death | PlayerActivity::Drown) {
			s.ts = TimeState::Stopped;
			s.events.push(GameEvent::GameOver { player: s.ps.ehandle });
		}
	}
}
