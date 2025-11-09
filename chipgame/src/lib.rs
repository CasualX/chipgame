
pub mod editor;
pub mod fx;
pub mod menu;
pub mod play;
pub mod data;
pub mod config;
pub mod render;

pub enum FileSystem {
	Paks(paks::FileReader, paks::Key),
	StdFs(std::path::PathBuf),
}
impl FileSystem {
	pub fn read_to_string(&self, path: &str) -> std::io::Result<String> {
		match self {
			FileSystem::Paks(paks, key) => {
				let desc = paks.find_desc(path.as_bytes()).ok_or_else(|| std::io::Error::from(std::io::ErrorKind::NotFound))?;
				let data = paks.read_data(&desc, key)?;
				String::from_utf8(data).map_err(|_| std::io::Error::from(std::io::ErrorKind::InvalidData))
			}
			FileSystem::StdFs(base) => std::fs::read_to_string(base.join(path)),
		}
	}
	pub fn read(&self, path: &str) -> std::io::Result<Vec<u8>> {
		match self {
			FileSystem::Paks(paks, key) => {
				let desc = paks.find_desc(path.as_bytes()).ok_or_else(|| std::io::Error::from(std::io::ErrorKind::NotFound))?;
				let data = paks.read_data(&desc, key)?;
				Ok(data)
			}
			FileSystem::StdFs(base) => std::fs::read(base.join(path)),
		}
	}
	/// Reads and decompresses a file.
	///
	/// If the file is in a paks archive, it will be decompressed using [chipty::decompress].
	pub fn read_compressed(&self, path: &str) -> std::io::Result<Vec<u8>> {
		match self {
			FileSystem::Paks(paks, key) => {
				let desc = paks.find_desc(path.as_bytes()).ok_or_else(|| std::io::Error::from(std::io::ErrorKind::NotFound))?;
				let data = paks.read_data(&desc, key)?;
				Ok(chipty::decompress(&data))
			}
			FileSystem::StdFs(base) => std::fs::read(base.join(path)),
		}
	}
}
