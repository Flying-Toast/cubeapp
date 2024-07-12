fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=resources/");

    let input_dir = "resources";
    let filenames = std::fs::read_dir(input_dir)
        .unwrap()
        .map(|ent| ent.unwrap().file_name())
        .filter(|name| name.to_str().unwrap().ends_with(".blp"))
        .map(|name| format!("{input_dir}/{}", name.to_str().unwrap()));

    let blp_out = std::process::Command::new("blueprint-compiler")
        .arg("batch-compile")
        .arg("resources") // output dir
        .arg(input_dir)
        .args(filenames)
        .output()
        .expect("failed to compile ui files; is `blueprint-compiler` installed?");
    if blp_out.status.code().unwrap() != 0 {
        panic!(
            "Blueprint compilation failed:\n{}",
            std::str::from_utf8(&blp_out.stdout).unwrap(),
        );
    }

    glib_build_tools::compile_resources(
        &["resources"],
        "resources/resources.gresource.xml",
        "PuzzleTime.gresource",
    );
}
