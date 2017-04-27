#[macro_use] extern crate plumbum;

use plumbum::Local;
use plumbum::errors::*;

pub fn main() {
    let local = Local::new();
    let output = local.bin("ls")
        .expect("command not found on path")
        .exec();
    println!("{:?}", output);

    let mut ls = local.bin("ls").unwrap();
    ls.arg("-l");
    let mut grep = local.bin("grep").unwrap();
    grep.arg("src");

    let out = pipe!(ls | grep);
    println!("{:?}", out);

    let mut cat = local.bin("cat").unwrap();
    cat.arg("Cargo.toml");
    let mut grep = local.bin("grep").unwrap();
    grep.args(&["walkdir"]);

    let out = pipe!(cat | grep);
    println!("{:?}", out);

    let out = pipe!(cat);
    println!("{:?}", out);
}
