#![recursion_limit = "1024"]
#[macro_use] extern crate error_chain;
extern crate walkdir;

pub mod errors {
    use std::io;
    error_chain! {
        foreign_links {
            Io(io::Error);
        }
        errors {
            Raw(s: String) {
                description("An error occurred")
                display("Error occurred: {:?}", s)
            }
        }
    }
    impl<'a> From<&'a String> for ErrorKind {
        fn from(s: &'a String) -> Self {
            ErrorKind::Raw(s.to_owned())
        }
    }
}

use walkdir::{WalkDir, WalkDirIterator};

use errors::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{self, Command, ExitStatus, Stdio};
use std::env;
use std::io::{Read, Write};


#[macro_export]
macro_rules! pipe {
    ($($com:ident)|*) => {
        {
            let o: Result<$crate::Output> = (||{
                let mut prev: Option<$crate::Output> = None;
                $(
                    {
                    let out: $crate::Output = $crate::Executable::exec(&mut $com, prev)
                        .chain_err(|| "Failed exec")?;
                    prev = Some(out);
                    }
                )*
                return Ok(prev.unwrap());
            })();
            o
        }
    };
}


pub trait Executable {
    fn exec(&mut self, input: Option<Output>) -> Result<Output>;
}


#[derive(Debug)]
pub struct Output {
    pub stdout: String,
    pub stderr: String,
}
impl Output {
    fn from_process_output(out: &str, err: &str) -> Self {
        Output {
            stdout: String::from(out),
            stderr: String::from(err),
        }
    }
}


#[derive(Debug)]
pub struct Bin {
    name: String,
    path: PathBuf,
    args: Vec<String>,
    command: Command,
}
impl Bin {
    pub fn new(name: &str, path: &Path) -> Self {
        let path = PathBuf::from(path);
        Bin {
            name: name.into(),
            command: Command::new(&path),
            path: path,
            args: vec![],
        }
    }
    pub fn arg(&mut self, arg: &str) -> &mut Self {
        self.args.push(arg.into());
        self.command.arg(arg);
        self
    }
    pub fn args(&mut self, args: &[&str]) -> &mut Self {
        for a in args.iter() {
            self.args.push(a.to_string());
            self.command.arg(a);
        }
        self
    }
    pub fn exec(&mut self) -> Result<Output> {
        Executable::exec(self, None)
    }
}
impl Executable for Bin {
    fn exec(&mut self, input: Option<Output>) -> Result<Output> {
        let p = self.command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to spawn proc");

        if let Some(input) = input {
            if !input.stderr.is_empty() {
                bail!(input.stderr);
            }
            p.stdin.unwrap().write_all(input.stdout.as_bytes())
                .expect("Failed writing to stdin");
        }
        let debug = format!("{:?}", self);
        let mut stdout = String::new();
        let mut stderr = String::new();
        p.stdout.unwrap().read_to_string(&mut stdout)
            .expect("Error reading stdout");
        p.stderr.unwrap().read_to_string(&mut stderr)
            .expect("Error reading stderr");
        Ok(Output::from_process_output(&stdout, &stderr))
    }
}


#[derive(Debug)]
pub struct Local {
    env: HashMap<String, String>,
    bins: HashMap<String, PathBuf>,
}
impl Local {
    pub fn new() -> Self {
        let path = env::var("PATH").expect("No `PATH` env-var found...");
        let path = path.split(':').collect::<Vec<_>>();
        let bins = path.iter().fold(HashMap::new(), |mut acc, p| {
            merge_maps(&mut acc, find_bins_in_path(PathBuf::from(p).as_path()));
            acc
        });
        let env_ = env::vars().collect();
        Local {
            env: env_,
            bins: bins,
        }
    }
    pub fn bin(&self, name: &str) -> Option<Bin> {
        self.bins.get(name)
            .map(|p| Bin::new(name, p))
    }
}


fn merge_maps(map: &mut HashMap<String, PathBuf>, more: HashMap<String, PathBuf>) {
    for (k, v) in more.into_iter() {
        map.insert(k, v);
    }
}


fn find_bins_in_path(p: &Path) -> HashMap<String, PathBuf> {
    let mut m = HashMap::new();
    let walker = WalkDir::new(p).follow_links(true).into_iter();
    for entry in walker.filter_entry(|e| e.file_name().to_str().is_some()) {
        if entry.is_err() { continue; }
        let entry = entry.unwrap();
        m.insert(entry.file_name().to_str().unwrap().to_string(), entry.path().to_path_buf());
    }
    m
}
