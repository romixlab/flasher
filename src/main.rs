use std::process::{Command, Stdio};
use std::{env, fmt};
use std::path::PathBuf;
use std::io::{Write, BufRead, BufReader};
use std::fs::File;
use termion::{color, style};
use std::fmt::Formatter;
use std::error::Error;

struct FlasherError(String);

impl fmt::Display for FlasherError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}{}", color::Fg(color::Red), self.0, style::Reset)
    }
}

impl fmt::Debug for FlasherError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl Error for FlasherError {}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() <= 1 {
        eprintln!("{}usage: jlink_flasher path_to.bin [jlink serial]{}", color::Fg(color::Blue), style::Reset);
        return Err(Box::new(FlasherError("wrong arguments".into())));
    }

    let current_dir = env::current_dir()?;
    let elf_path = &args[1];
    let mut target_dir = PathBuf::new();
    target_dir.push(&current_dir);
    target_dir.push(&elf_path);
    let elf_name = String::from(target_dir.file_name().unwrap().to_str().unwrap());
    target_dir.pop();

    let mut config_path = target_dir.clone();
    let pop_count: u8 =
    if config_path.ends_with("examples") {
        4
    } else {
        3
    };
    for _ in 0..pop_count {
        config_path.pop();
    }
    config_path.push("flasher.conf");
    let config = match File::open(config_path) {
        Ok(c) => c,
        Err(e) => {
            println!("{}{}", color::Fg(color::Yellow), e);
            return Err(Box::new(FlasherError("Config file missing?".into())));
        }
    };
    let config_lines = BufReader::new(config).lines();

    let mut objcopy = Command::new("arm-none-eabi-objcopy");
    objcopy.current_dir(&target_dir);
    objcopy.arg("-O");
    objcopy.arg("binary");
    objcopy.arg(&elf_name);
    objcopy.arg(elf_name.clone() + ".bin");
    //println!("{:?}", objcopy_cmd);
    match objcopy.status() {
        Ok(s) => {
            if !s.success() {
                return Err(Box::new(FlasherError("objcopy returned non-zero, internal error".into())));
            }
        },
        Err(e) => {
            println!("{}{}\n", color::Fg(color::Yellow), e);
            return Err(Box::new(FlasherError("Maybe arm-none-eabi-objcopy is not installed or not in path?".into())));
        }
    };
    println!("{}Created: {}.bin{}", color::Fg(color::Green), elf_name, style::Reset);

    let mut jlinkexe = Command::new("JLinkExe");
    // if args.len() == 3 {
    //     let serial = &args[2];
    //     println!("Serial provided: {}", serial);
    //     jlinkexe.arg("USB");
    //     jlinkexe.arg(serial);
    // }
    for i in 2..args.len() {
        jlinkexe.arg(&args[i]);
    }
    jlinkexe.stdin(Stdio::piped());
    let mut jlinkexe = match jlinkexe.spawn() {
        Ok(j) => j,
        Err(e) => {
            println!("{}{}", color::Fg(color::Red), e);
            return Err(Box::new(FlasherError("JLinkExe is not installed or not in path?".into())));
        }
    };

    {
        let stdin = jlinkexe.stdin.as_mut().unwrap();
        for l in config_lines {
            if let Ok(l) = l {
                if l.contains("{bin_path}") {
                    let mut bin_path = target_dir.clone();
                    bin_path.push(elf_name.clone() + ".bin");
                    stdin.write_all(l.replace("{bin_path}", bin_path.to_str().unwrap()).as_bytes())?;
                    stdin.write_all("\n".as_bytes())?;
                } else {
                    stdin.write_all(l.as_bytes())?;
                    stdin.write_all("\n".as_bytes())?;
                }
            }
        }
    }

    let jle_exit = jlinkexe.wait()?;
    if jle_exit.success() {
        println!("{}JLinkExe exited with 0", color::Fg(color::Green))
    } else {
        println!("{}JLinkExe exited with {:?}", color::Fg(color::Red), jle_exit);
    }

    Ok(())
}