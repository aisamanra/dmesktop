#[macro_use] extern crate failure;
extern crate xdg;
extern crate xdg_desktop;

use failure::Error;
use xdg_desktop::{DesktopEntry, EntryType};

use std::io::{Write};
use std::process::{self,Command,Stdio};
use std::os::unix::process::CommandExt;

fn ensure_rofi() -> Result<(), Error> {
    let _ = Command::new("which")
        .args(&["rofi"])
        .output()
        .map_err(|_| format_err!("could not find `rofi'"))?;
    Ok(())
}

/// Given a list of strings, we provide them to rofi and return back
/// the one which the user chose (or an empty string, if the user
/// chose nothing)
fn rofi_choose<'a, I>(choices: I) -> Result<String, Error>
    where I: Iterator<Item=&'a String>
{
    let mut rofi = Command::new("rofi")
        .args(&["-dmenu", "-i", "-l", "10"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    {
        let stdin = rofi.stdin.as_mut().unwrap();
        for c in choices.into_iter() {
            stdin.write(c.as_bytes())?;
            stdin.write(b"\n")?;
        }
    }

    let output = rofi.wait_with_output()?;
    Ok(String::from_utf8(output.stdout)?.trim().to_owned())
}

fn is_not_metavar(s: &&str) -> bool {
    !(s.starts_with("%") && s.len() == 2)
}

fn run_command(cmd: &Option<String>) -> Result<(), Error> {
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

fn fetch_entries() -> Result<Vec<xdg_desktop::DesktopEntry>, Error> {
    let mut entries = Vec::new();
    for f in xdg::BaseDirectories::new()?.list_data_files("applications") {
        if f.extension().map_or(false, |e| e == "desktop") {
            let mut f = std::fs::File::open(f)?;
            match xdg_desktop::DesktopEntry::from_file(&mut f) {
                Ok(e) => if e.is_application() {
                    entries.push(e);
                },
                _ => (),
            }
        }
    }
    Ok(entries)
}

fn main() -> Result<(), Error> {
    ensure_rofi()?;

    let entries = fetch_entries()?;

    let choice = rofi_choose(entries.iter()
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
