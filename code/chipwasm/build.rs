use std::env;
use std::path::PathBuf;

fn main() {
	let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
	let repo_root = manifest_dir.join("../..").canonicalize().unwrap();
	let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));

	println!("cargo:rerun-if-changed={}", repo_root.join("data").display());
	for levelset in chipbundle::CURATED_BUNDLED_LEVELSETS {
		println!("cargo:rerun-if-changed={}", repo_root.join("levelsets").join(levelset).display());
	}

	chipbundle::generate_bundled_assets(&repo_root, &out_dir).expect("failed to generate bundled assets");
}
