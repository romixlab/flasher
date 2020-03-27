use std::process::{Command, Stdio};
use std::env;
use std::path::PathBuf;
use std::io::{Write, BufRead, BufReader};
use std::fs::File;
use termion::{color, style};

fn main() -> Result<(), std::io::Error> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("{}\nPlease provide the path to an elf file.", color::Fg(color::Red));
        return Ok(())
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
            println!("{}{}\nConfig file missing?", color::Fg(color::Red), e);
            return Ok(())
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
                println!("{}objcopy returned non-zero, internal error", color::Fg(color::Red));
                return Ok(())
            }
        },
        Err(e) => {
            println!("{}Error: {}\nMaybe arm-none-eabi-objcopy is not installed or not in path?", color::Fg(color::Red), e);
            return Ok(())
        }
    };
    println!("{}Created: {}.bin{}", color::Fg(color::Green), elf_name, style::Reset);

    let mut jlinkexe = Command::new("JLinkExe");
    jlinkexe.stdin(Stdio::piped());
    let mut jlinkexe = match jlinkexe.spawn() {
        Ok(j) => j,
        Err(e) => {
            println!("{}Error: {}\nJLinkExe is not installed or not in path?", color::Fg(color::Red), e);
            return Ok(())
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