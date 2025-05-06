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
#[derive(Clone)]
pub struct PlayerState {
	pub ehandle: EntityHandle,

	/// Player input manager.
	pub inbuf: InputBuffer,

	/// Current player activity.
	pub activity: PlayerActivity,
	/// True if previous movement was involuntary.
	pub forced_move: bool,
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

	pub cheats_enable: bool,
	pub cs_wtw: CodeSequenceState,
	pub cs_giveall: CodeSequenceState,
	pub cs_inftime: CodeSequenceState,
	pub cs_win: CodeSequenceState,
}

impl Default for PlayerState {
	fn default() -> PlayerState {
		PlayerState {
			ehandle: EntityHandle::INVALID,
			inbuf: InputBuffer::default(),
			activity: PlayerActivity::Walking,
			forced_move: false,
			last_step_dir: None,
			steps: 0,
			bonks: 0,
			attempts: 0,
			chips: 0,
			keys: [0; 4],
			flippers: false,
			fire_boots: false,
			ice_skates: false,
			suction_boots: false,
			dev_wtw: false,
			cheats_enable: false,
			cs_wtw: CodeSequenceState::default(),
			cs_giveall: CodeSequenceState::default(),
			cs_inftime: CodeSequenceState::default(),
			cs_win: CodeSequenceState::default(),
		}
	}
}

pub(super) fn ps_input(s: &mut GameState, input: &Input) {
	s.ps.inbuf.handle(Compass::Left,  input.left,  s.input.left);
	s.ps.inbuf.handle(Compass::Right, input.right, s.input.right);
	s.ps.inbuf.handle(Compass::Up,    input.up,    s.input.up);
	s.ps.inbuf.handle(Compass::Down,  input.down,  s.input.down);

	if s.ps.cheats_enable {
		if input.left && !s.input.left {
			ps_nextcs(s, Button::Left);
		}
		if input.right && !s.input.right {
			ps_nextcs(s, Button::Right);
		}
		if input.up && !s.input.up {
			ps_nextcs(s, Button::Up);
		}
		if input.down && !s.input.down {
			ps_nextcs(s, Button::Down);
		}
		if input.a && !s.input.a {
			ps_nextcs(s, Button::A);
		}
		if input.b && !s.input.b {
			ps_nextcs(s, Button::B);
		}
		if input.start && !s.input.start {
			ps_nextcs(s, Button::Start);
		}
		if input.select && !s.input.select {
			ps_nextcs(s, Button::Select);
		}
	}
}

fn ps_nextcs(s: &mut GameState, btn: Button) {
	if s.ps.cs_wtw.next(btn, &CODE_WTW) {
		s.ps.dev_wtw = !s.ps.dev_wtw;
	}
	if s.ps.cs_giveall.next(btn, &CODE_GIVEALL) {
		s.ps.flippers = true;
		s.ps.fire_boots = true;
		s.ps.ice_skates = true;
		s.ps.suction_boots = true;
		s.ps.keys = [99; 4];
	}
	if s.ps.cs_inftime.next(btn, &CODE_INFTIME) {
		s.field.time_limit = 0;
	}
	if s.ps.cs_win.next(btn, &CODE_WIN) {
		ps_activity(s, PlayerActivity::Win);
	}
}

pub(super) fn ps_activity(s: &mut GameState, activity: PlayerActivity) {
	if s.ps.activity != activity {
		s.ps.activity = activity;
		s.events.fire(GameEvent::PlayerActivity { player: () });

		if activity.is_game_over() {
			s.ts = TimeState::Paused;
		}

		match activity {
			PlayerActivity::Drowned => {
				s.events.fire(GameEvent::GameOver { player: () });
				s.events.fire(GameEvent::SoundFx { sound: SoundFx::WaterSplash });
			}
			PlayerActivity::Burned => {
				s.events.fire(GameEvent::GameOver { player: () });
				s.events.fire(GameEvent::SoundFx { sound: SoundFx::FireWalking });
			}
			PlayerActivity::Bombed => {
				s.events.fire(GameEvent::GameOver { player: () });
				// Already fired by the Bomb entity!
				// s.events.fire(GameEvent::SoundFx { sound: SoundFx::BombExplosion });
			}
			PlayerActivity::OutOfTime => {
				s.events.fire(GameEvent::GameOver { player: () });
				s.events.fire(GameEvent::SoundFx { sound: SoundFx::GameOver });
			}
			PlayerActivity::Collided => {
				s.events.fire(GameEvent::GameOver { player: () });
				s.events.fire(GameEvent::SoundFx { sound: SoundFx::GameOver });
			}
			PlayerActivity::Eaten => {
				s.events.fire(GameEvent::GameOver { player: () });
				s.events.fire(GameEvent::SoundFx { sound: SoundFx::GameOver });
			}
			PlayerActivity::Win => {
				s.events.fire(GameEvent::GameWin { player: () });
				s.events.fire(GameEvent::SoundFx { sound: SoundFx::GameWin });
			}
			_ => (),
		}
	}
}
