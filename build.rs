use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // This is not something you should be doing
    // in a build rs file
    // use npm or something
    // I am bad at computers
    let out_dir = env::var("OUT_DIR").expect("Failed to get OUT_DIR env var");
    let contents: String =
        ureq::get("https://cdnjs.cloudflare.com/ajax/libs/nipplejs/0.9.0/nipplejs.min.js")
            .call()
            .expect("Failed to query cloudflare cdn")
            .into_string()
            .expect("Failed to convert response to string");
    let path = Path::new(&out_dir).join("nipplejs.min.js");
    fs::write(path, contents).expect("Failed to write to file");
}
