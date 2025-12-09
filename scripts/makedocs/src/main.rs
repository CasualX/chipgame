use std::{fs, path};

fn main() {
	render_content(
		include_str!("../../../readme.md"),
		include_str!("template.html"),
		path::Path::new("target/publish/readme.html"),
	);

	// Copy docs folder
	copy_files(path::Path::new("docs"), path::Path::new("target/publish/docs"));
}

fn render_content(markdown: &str, template: &str, target: &path::Path) {
	// Trusted markdown options
	let mut opts = markdown::Options::gfm();
	opts.parse.constructs.frontmatter = false;
	opts.parse.constructs.html_flow = true;
	opts.parse.constructs.html_text = true;
	opts.compile.allow_dangerous_html = true;
	opts.compile.allow_dangerous_protocol = true;
	opts.compile.allow_any_img_src = true;
	opts.compile.gfm_tagfilter = false;
	let html = markdown::to_html_with_options(markdown, &opts).unwrap();

	let html = template.replace("<!-- CONTENT -->", &html);

	fs::write(target, html).unwrap();
}

fn copy_files(src: &path::Path, dest: &path::Path) {
	if !dest.exists() {
		fs::create_dir_all(dest).unwrap();
	}
	for entry in fs::read_dir(src).unwrap() {
		let entry = entry.unwrap();
		let file_type = entry.file_type().unwrap();
		let dest_path = dest.join(entry.file_name());
		if file_type.is_dir() {
			copy_files(&entry.path(), &dest_path);
		}
		else if file_type.is_file() {
			fs::copy(entry.path(), dest_path).unwrap();
		}
	}
}
