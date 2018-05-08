#[macro_use] extern crate failure;
extern crate xdg_desktop;

use failure::Error;
use xdg_desktop::{DesktopEntry, EntryType};

use std::io::{Write};
use std::process::{self,exit,Command,Stdio};
use std::os::unix::process::CommandExt;

fn ensure_dmenu() -> Result<(), Error> {
    let _ = Command::new("which")
        .args(&["dmenu"])
        .output()
        .map_err(|_| format_err!("could not find `dmenu'"))?;
    Ok(())
}

/// Given a list of strings, we provide them to dmenu and return back
/// the one which the user chose (or an empty string, if the user
/// chose nothing)
fn dmenu_choose<'a, I>(choices: I) -> Result<String, Error>
    where I: Iterator<Item=&'a String>
{
    let mut dmenu = Command::new("dmenu")
        .args(&["-i", "-l", "10"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    {
        let stdin = dmenu.stdin.as_mut().unwrap();
        for c in choices.into_iter() {
            stdin.write(c.as_bytes())?;
            stdin.write(b"\n")?;
        }
    }

    let output = dmenu.wait_with_output()?;
    Ok(String::from_utf8(output.stdout)?.trim().to_owned())
}

fn is_not_metavar(s: &&str) -> bool {
    !(s.starts_with("%") && s.len() == 2)
}

fn run_command(cmd: &Option<String>) -> Result<(), Error> {
    println!("runnin");
    if let &Some(ref cmd) = cmd {
        let mut iter = cmd.split_whitespace();
        process::Command::new(iter.next().unwrap())
            .args(iter.filter(is_not_metavar))
            .exec();
    } else {
        Err(format_err!("No command specified in file!"))?;
    }
    Ok(())
}

fn run() -> Result<(), Error> {
    ensure_dmenu()?;
    let mut entries = Vec::new();
    for f in std::fs::read_dir("/usr/share/applications")? {
        let f = f?;
        if f.file_type()?.is_file() && f.path().extension().map_or(false, |e| e == "desktop") {
            let mut f = std::fs::File::open(f.path())?;
            match xdg_desktop::DesktopEntry::from_file(&mut f) {
                Ok(e) => if e.is_application() {
                    entries.push(e);
                },
                _ => (),
            }
        }
    }

    let choice = dmenu_choose(entries.iter()
                              .map(|e| &e.info.name))?;
    println!("Choice is {}", choice);

    match entries.iter().find(|e| e.info.name == choice) {
        Some(&DesktopEntry {
            typ: EntryType::Application(ref app),
            info: _,
        }) => run_command(&app.exec)?,
        _ => (),
    }
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error running dmesktop: {}", e);
        exit(99);
    }
}
