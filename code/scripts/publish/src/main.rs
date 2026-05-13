use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

const README_TEMPLATE: &str = include_str!("template.html");

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..").canonicalize()?;
	let current_dir = env::current_dir()?;
	let output_dir = output_dir(&current_dir);

	prepare_output_dir(&output_dir)?;
	build_desktop_binaries(&repo_root)?;
	stage_release_files(&repo_root, &output_dir)?;
	render_docs(&repo_root, &output_dir)?;

	println!("Wrote {}", output_dir.display());
	Ok(())
}

fn output_dir(current_dir: &Path) -> PathBuf {
	match env::args_os().nth(1) {
		Some(path) => {
			let path = PathBuf::from(path);
			if path.is_absolute() { path } else { current_dir.join(path) }
		}
		None => current_dir.join("target/publish"),
	}
}

fn prepare_output_dir(output_dir: &Path) -> io::Result<()> {
	if output_dir.exists() {
		let error = format!("output directory {} already exists; remove it first and rerun publish", output_dir.display());
		return Err(io::Error::new(io::ErrorKind::AlreadyExists, error));
	}
	fs::create_dir_all(output_dir.join("save"))?;
	Ok(())
}

fn build_desktop_binaries(repo_root: &Path) -> Result<(), Box<dyn std::error::Error>> {
	let cargo = env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
	let status = Command::new(cargo)
		.current_dir(repo_root)
		.args(["build", "--release", "--bin", "chipplay", "--bin", "chipedit"])
		.status()?;
	if !status.success() {
		return Err(format!("cargo build failed with status {status}").into());
	}
	Ok(())
}

fn stage_release_files(repo_root: &Path, output_dir: &Path) -> io::Result<()> {
	let release_dir = repo_root.join("target/release");
	let chipplay = format!("chipplay{}", env::consts::EXE_SUFFIX);
	let chipedit = format!("chipedit{}", env::consts::EXE_SUFFIX);
	copy_file(release_dir.join(&chipplay), output_dir.join(&chipplay))?;
	copy_file(release_dir.join(&chipedit), output_dir.join(&chipedit))?;
	copy_if_exists(release_dir.join("chipplay.pdb"), output_dir.join("chipplay.pdb"))?;
	copy_if_exists(release_dir.join("chipedit.pdb"), output_dir.join("chipedit.pdb"))?;
	copy_file(repo_root.join("chipdx.ini"), output_dir.join("chipdx.ini"))?;
	chipbundle::generate_bundled_assets(repo_root, output_dir)?;
	Ok(())
}

fn render_docs(repo_root: &Path, output_dir: &Path) -> io::Result<()> {
	let readme = fs::read_to_string(repo_root.join("readme.md"))?;
	let mut opts = markdown::Options::gfm();
	opts.parse.constructs.frontmatter = false;
	opts.parse.constructs.html_flow = true;
	opts.parse.constructs.html_text = true;
	opts.compile.allow_dangerous_html = true;
	opts.compile.allow_dangerous_protocol = true;
	opts.compile.allow_any_img_src = true;
	opts.compile.gfm_tagfilter = false;

	let html = markdown::to_html_with_options(&readme, &opts)
		.map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err.to_string()))?;
	fs::write(output_dir.join("readme.html"), README_TEMPLATE.replace("<!-- CONTENT -->", &html))?;
	copy_tree(&repo_root.join("docs"), &output_dir.join("docs"))?;
	Ok(())
}

fn copy_tree(src: &Path, dest: &Path) -> io::Result<()> {
	fs::create_dir_all(dest)?;
	let mut entries: Vec<_> = fs::read_dir(src)?.collect::<Result<_, _>>()?;
	entries.sort_by_key(|entry| entry.file_name());
	for entry in entries {
		let entry_path = entry.path();
		let dest_path = dest.join(entry.file_name());
		if entry.file_type()?.is_dir() {
			copy_tree(&entry_path, &dest_path)?;
		}
		else {
			copy_file(entry_path, dest_path)?;
		}
	}
	Ok(())
}

fn copy_file(src: impl AsRef<Path>, dest: impl AsRef<Path>) -> io::Result<u64> {
	let dest = dest.as_ref();
	if let Some(parent) = dest.parent() {
		fs::create_dir_all(parent)?;
	}
	fs::copy(src, dest)
}

fn copy_if_exists(src: impl AsRef<Path>, dest: impl AsRef<Path>) -> io::Result<()> {
	let src = src.as_ref();
	if src.exists() {
		copy_file(src, dest)?;
	}
	Ok(())
}
