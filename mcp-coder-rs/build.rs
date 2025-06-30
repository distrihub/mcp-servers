use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    
    let dir: PathBuf = ["tree-sitter-rust", "src"].iter().collect();
    if dir.exists() {
        cc::Build::new()
            .include(&dir)
            .file(dir.join("parser.c"))
            .compile("tree-sitter-rust");
    }

    let dir: PathBuf = ["tree-sitter-javascript", "src"].iter().collect();
    if dir.exists() {
        cc::Build::new()
            .include(&dir)
            .file(dir.join("parser.c"))
            .compile("tree-sitter-javascript");
    }

    let dir: PathBuf = ["tree-sitter-python", "src"].iter().collect();
    if dir.exists() {
        cc::Build::new()
            .include(&dir)
            .file(dir.join("parser.c"))
            .compile("tree-sitter-python");
    }
}