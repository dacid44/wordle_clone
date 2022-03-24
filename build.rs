use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let word_list_str = fs::read_to_string("words5.txt").unwrap().trim().to_string();

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("word_list.rs");
    #[rustfmt::skip] // rustfmt wants to put the final .unwrap() on a new line
    fs::write(
        &dest_path,
        format!(
            "pub static WORD_LIST: [&str; {}] = [\n{}\n];",
            word_list_str.lines().count(),
            word_list_str
                .lines()
                .map(|x| format!("    \"{}\",", x.to_uppercase()))
                .collect::<Vec<String>>()
                .join("\n")
        ),
    ).unwrap();

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=words5.txt");
}
