use anyhow::Result;

use anyhow::bail;
use lazy_static::lazy_static;
use log::info;
use regex::Regex;
use regex::RegexSet;
use std::str;
use xshell::{cmd, Shell};

#[derive(Debug)]
pub struct Monitor {
    name: String,
    x_pos: u32,
    y_pos: u32,
    width: u32,
    height: u32,
    primary: bool,
}

#[derive(Debug)]
pub struct Monitors(Vec<Monitor>);

pub fn get_monitors() -> Result<Monitors> {
    let sh = Shell::new()?;
    let binding = cmd!(sh, "xrandr").output()?;
    let stdout = str::from_utf8(&binding.stdout).unwrap();
    lazy_static! {
        static ref RE: Regex =
            Regex::new(r#"(?:(\S+) connected (primary)? ?(\d+)x(\d+)\+(\d+)\+(\d+))"#,).unwrap();
    }
    let caps = RE.captures_iter(stdout);
    let displays: Vec<Monitor> = caps
        .map(|cap| Monitor {
            name: cap[1].to_string(),
            primary: cap.get(2).map_or(false, |m| m.as_str() == "primary"),
            x_pos: cap[5].parse().unwrap(),
            y_pos: cap[6].parse().unwrap(),
            width: cap[3].parse().unwrap(),
            height: cap[4].parse().unwrap(),
        })
        .collect();
    Ok(Monitors(displays))
    // bail!("Could not parse xrandr with regex \n{stdout}");
}
