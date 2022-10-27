use std::{path::{Path, PathBuf}, str::FromStr};

use chrono::{DateTime, Local, TimeZone};
use lazy_static::lazy_static;
use regex::Regex;
use serde_with::DeserializeFromStr;


#[macro_export]
macro_rules! or2s {
    ($expr:expr) => {
        $expr.or_else(|err| Err(format!("{err:?}")))
    };
}

#[macro_export]
macro_rules! osstr2str {
    ($expr:expr) => {
        {
            let expr = $expr;
            expr
            .to_str()
            .ok_or(format!("NonUtf8 OsStr {expr:?}"))
        }
    };
}


pub type Result<T> = std::result::Result<T, String>;


#[derive(Debug, DeserializeFromStr)]
pub struct RelaDateTime(
    pub DateTime<Local>
);




impl FromStr for RelaDateTime {
    type Err = String;

    fn from_str(s: &str) -> Result<Self> {
        lazy_static! {
            static ref REG_DATETIME1: Regex = 
                Regex::new(r"(?P<y>\d{4})-(?P<m>\d{2})-(?P<d>\d{2})").unwrap();
        }

        if let Some(cap) = REG_DATETIME1.captures(s) {
            let y = or2s!(cap.name("y").unwrap().as_str().parse())?;
            let m = or2s!(cap.name("m").unwrap().as_str().parse())?;
            let d = or2s!(cap.name("d").unwrap().as_str().parse())?;

            return Ok(Self(
                Local.ymd(y, m, d)
                .and_hms(0, 0, 0)
            ))
        }

        Err(format!("No matched dt format for {s}"))
    }
}



////////////////////////////////////////////////////////////////////////////////
//// Function

pub fn file_stem<P: AsRef<Path>>(p: P) -> Result<String> {
    match p.as_ref().file_stem() {
        Some(osstr) => {
            Ok(osstr2str!(osstr)?.to_owned())
        }
        None => {
            Err(format!("NO file stam found for {:?}", p.as_ref()))
        },
    }
}


pub fn file_name<P: AsRef<Path>>(p: P) -> Result<String> {
    match p.as_ref().file_name() {
        Some(osstr) => {
            Ok(osstr2str!(osstr)?.to_owned())
        }
        None => {
            Err(format!("NO file name found for {:?}", p.as_ref()))
        },
    }
}


pub fn read_to_string<P: AsRef<Path>>(p: P) -> Result<String> {
    std::fs::read_to_string(p)
        .or_else(|err| Err(format!("{err:?}")))
}


pub fn is_dot_file<P: AsRef<Path>>(p: P) -> bool {
    if let Some(name) = p.as_ref().file_name() {
        name.to_string_lossy().starts_with(".")
    }
    else {
        false
    }
}

pub fn mkdirs<P: AsRef<Path>>(p: P) -> Result<()> {
    or2s!(std::fs::create_dir_all(p))
}


pub fn pwd() -> Result<PathBuf> {
    or2s!(std::env::current_dir())
}

pub fn shorten_path(p: &Path) -> Result<PathBuf> {
    match pathdiff::diff_paths(p, pwd()?) {
        Some(res) => Ok(res),
        None => Ok(p.to_owned()),
    }
}


