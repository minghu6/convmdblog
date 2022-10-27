use std::{
    collections::HashMap,
    fs,
    path::Path,
};

use chrono::Datelike;
use lazy_static::lazy_static;
use serde_yaml::{Mapping, Value};

use super::{map_img_ref, Mapper};
use crate::{
    aux::{shorten_path, Result},
    or2s,
    reader::default::DefaultReader,
};

macro_rules! no_yaml_hdr {
    ($p:expr) => {
        return Err(format!("No yaml header from {:?}", $p))
    };
}

macro_rules! no_date_tag {
    ($p:expr) => {
        return Err(format!("No date tag on yaml header from {:?}", $p))
    };
}

macro_rules! no_title_tag {
    ($p:expr) => {
        return Err(format!("No title tag on yaml header from {:?}", $p))
    };
}

lazy_static! {
    static ref TAG_TO_CAT_MAP: HashMap<&'static str, Cat> = {
        let mut map = HashMap::new();

        let mut iter = vec![];

        // ALGS
        let algs = vec![
            "string",
            "string pattern match",
            "algorithm",
            "graph",
        ];
        iter.push((Cat::Algs, algs));

        // LANG
        let lang = vec![
            "c",
            "common lisp",
            "php",
            "haskell",
            "hy",
            "python",
            "compiler",
            "llvm"
        ];
        iter.push((Cat::Lang, lang));

        // OS
        let os = vec![
            "linux",
            "kernel",
            "fs",
            "shell",
            "bash",
            "sudo"
        ];
        iter.push((Cat::OS, os));

        // Net
        let net = vec![
            "ietf rfcs",
            "ietf"
        ];
        iter.push((Cat::Net, net));

        for (cat, targets) in iter {

            for tag in targets {
                map.insert(tag, cat);
            }
        }

        map
    };
}

pub enum JekyllMapper {
    Default,
}

/// Category
#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd)]
#[non_exhaustive]
enum Cat {
    Algs,
    Lang,
    OS,
    Net,
    #[default]
    Oth,
}

impl Mapper for JekyllMapper {
    fn mapping(&self, input: &Path, outdir: &Path) -> Result<()> {
        let handler = match self {
            JekyllMapper::Default => map_default_to_jekyll,
        } as fn(&Path, &Path) -> Result<()>;

        handler(&input.as_ref(), &outdir.as_ref())?;

        Ok(())
    }
}

// impl FromStr for Cat {
//     type Err = Infallible;

//     fn from_str(s: &str) -> Result<Self, Self::Err> {

//         let s = s.trim().to_lowercase();

//     }
// }

fn map_default_to_jekyll(input: &Path, outdir: &Path) -> Result<()> {
    let reader = DefaultReader::from_path(input)?;

    /* Common mapping */
    let text = map_img_ref(&reader.raw[reader.text_start..], "/assets/img")?;

    /* Specify the file name*/
    let yaml_hdr;
    match reader.yaml_hdr {
        Some(hdr) => {
            yaml_hdr = hdr;
        }
        None => no_yaml_hdr!(input),
    }

    let date_prefix;
    match yaml_hdr.date {
        Some(rela) => {
            date_prefix = format!(
                "{:04}-{:02}-{:02}",
                rela.0.year(),
                rela.0.month(),
                rela.0.day()
            )
        }
        None => no_date_tag!(input),
    }
    let file_title = reader.name_stem;

    let out = outdir.join(format!("{date_prefix}-{file_title}.md"));

    /* Specify the yaml header*/
    let text_title;
    match yaml_hdr.title {
        Some(title) => {
            text_title = title;
        }
        None => no_title_tag!(input),
    }

    let mut root_map = Mapping::new();

    root_map.insert(Value::String("title".to_owned()), Value::String(text_title));

    root_map.insert(Value::String("date".to_owned()), Value::String(date_prefix));

    root_map.insert(
        Value::String("layout".to_owned()),
        Value::String("post".to_owned()),
    );

    root_map.insert(Value::String("mathjax".to_owned()), Value::Bool(true));

    // root_map.insert(
    //     Value::String("tags".to_owned()),
    //     Value::Sequence(
    //         yaml_hdr.tags
    //         .into_iter()
    //         .map(|s| Value::String(s))
    //         .collect()
    //     )
    // );

    let cats = map_tags_to_cats(&yaml_hdr.tags);
    let cats_value = cats
        .into_iter()
        .map(|cat| Value::String(format!("{cat:?}").to_lowercase()))
        .collect();

    root_map.insert(
        Value::String("category".to_owned()),
        Value::Sequence(cats_value),
    );

    let yaml_text = or2s!(serde_yaml::to_string(&root_map))?;

    /* Open and write it */
    or2s!(fs::write(&out, format!("---\n{yaml_text}---\n{text}")))?;

    println!("write {}", shorten_path(&out)?.to_string_lossy());

    Ok(())
}

fn map_tags_to_cats<S: AsRef<str>>(tags: &[S]) -> Vec<Cat> {
    let mut catopt = None;

    for tag in tags {
        let tag = tag.as_ref().trim().to_lowercase();

        if let Some(cat) = TAG_TO_CAT_MAP.get(tag.as_str()) {
            if let Some(ref _cat) = catopt {
                if _cat != cat {
                    unreachable!("Multiple Category found! {cat:?}, {_cat:?}");
                }
            } else {
                catopt = Some(*cat);
            }
        }
    }

    vec![catopt.unwrap_or_default()]
}
