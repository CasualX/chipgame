use super::*;

impl cvar::IVisit for GameState {
	fn visit(&mut self, f: &mut dyn FnMut(&mut dyn cvar::INode)) {
		f(&mut cvar::Action("cheats.wtw!", |_, fd| {
			self.ps.dev_wtw = !self.ps.dev_wtw;
			_ = write!(fd, "Walk Through Walls: {}\n", if self.ps.dev_wtw { "Enabled" } else { "Disabled" });
		}));
		f(&mut cvar::Action("cheats.give!", |args, fd| {
			if args.is_empty() || args == "boots" {
				self.ps.flippers = true;
				self.ps.fire_boots = true;
				self.ps.ice_skates = true;
				self.ps.suction_boots = true;
				_ = write!(fd, "Given boots.\n");
			}
			if args.is_empty() || args == "keys" {
				self.ps.keys = [99; 4];
				_ = write!(fd, "Given keys.\n");
			}
		}));
		f(&mut cvar::Action("cheats.time!", |_, fd| {
			self.field.time_limit = 0;
			_ = write!(fd, "Time limit removed.\n");
		}));
	}
}
