use std::{
    io::{self, Write},
    str::FromStr, ops::Range,
};

use crate::filesystem::{DirectoryEntry, FsRead};

enum Command {
    Help,
    Ls,
    Cd(String),
    Rd(String, Range<usize>),
}

impl FromStr for Command {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.trim().split_ascii_whitespace();
        let command = parts.next().ok_or("Empty command")?;

        println!("{command}");

        if command == "help" {
            Ok(Self::Help)
        } else if command == "ls" {
            Ok(Self::Ls)
        } else if command == "cd" {
            let dir = parts.next().ok_or("cd must be followed by folder name")?;
            Ok(Self::Cd(dir.into()))
        } else if command == "rd" {
            let msg = "use help for rd usage";
            let file = parts.next().ok_or(msg)?;
            let from = parts.next().ok_or(msg)?.parse::<usize>().map_err(|_| msg)?;
            let to = parts.next().ok_or(msg)?.parse::<usize>().map_err(|_| msg)?;

            Ok(Command::Rd(file.into(), from..to))
        } else {
            Err("No such command".into())
        }
    }
}

pub struct Tui<FS: FsRead> {
    fs: FS,
    dir: FS::NodeHandle,
}

impl<FS: FsRead> Tui<FS> {
    pub fn new(mut fs: FS) -> Self {
        fs.init();
        let dir = fs.dir_root();
        Tui { fs, dir }
    }

    pub fn run(&mut self) {
        let mut line = String::new();
        let mut dir_data = Vec::new();

        loop {
            print!("> ");
            io::stdout().flush().unwrap();

            line.clear();
            io::stdin()
                .read_line(&mut line)
                .expect("Failed to read line");

            let cmd = match Command::from_str(&line) {
                Ok(cmd) => cmd,
                Err(msg) => {
                    println!("{}", msg);
                    Command::Help
                }
            };

            match cmd {
                Command::Help => {
                    println!("ls - list entries in the working directory");
                    println!("cd <FOLDER> - list entries in the working directory");
                    println!("rd <FILE> <FROM> <TO>");
                    println!("help - display this message");
                }
                Command::Ls => {
                    self.fs.dir_read(&self.dir, &mut dir_data).unwrap();
                    for (i, entry) in dir_data.iter().enumerate() {
                        println!("{}) {}", i + 1, entry.name.to_string_lossy().as_ref());
                    }
                }
                Command::Cd(name) => {
                    self.fs.dir_read(&self.dir, &mut dir_data).unwrap();
                    match dir_data.iter().find(|entry| entry.name == name.as_str()) {
                        Some(DirectoryEntry { handle, .. }) => self.dir = handle.clone(),
                        None => {
                            println!("No such directory");
                        }
                    }
                }
                Command::Rd(name, range) => {
                    self.fs.dir_read(&self.dir, &mut dir_data).unwrap();
                    let handle = match dir_data.iter().find(|entry| entry.name == name.as_str()) {
                        Some(DirectoryEntry { handle, .. }) => handle,
                        None => {
                            println!("No such file");
                            continue;
                        }
                    };

                    let file_ctx = match self.fs.file_open(&handle) {
                        Ok(cx) => cx,
                        Err(e) => {
                            println!("{:?}", e);
                            continue
                        }
                    };

                    let mut buf = vec![0u8; range.len()];
                    if let Err(e) = self.fs.file_read(handle, &file_ctx, range, &mut buf) {
                        println!("{:?}", e);
                        continue;
                    }

                    for (i, byte) in buf.iter().enumerate() {
                        print!("{:02x} ", byte);
                        if i != 0 {
                            if (i + 1) % 8 == 0 {
                                print!(" ");
                            }
                            if (i + 1) % 32 == 0 {
                                println!("");
                            }
                        }
                    }
                    println!("");
                }
            }
        }
    }
}

impl<FS: FsRead> Drop for Tui<FS> {
    fn drop(&mut self) {
        self.fs.destroy();
    }
}
