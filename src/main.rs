use std::{path::PathBuf, str::FromStr};

use clap::Parser;
use convmdblog::{
    mapper::{jekyll::JekyllMapper, Mapper},
    aux::{Result, mkdirs}, batcher::syn_walk
};


#[derive(Debug, Default)]
enum Mat {
    #[default]
    Default,
    Jekyll,
}

#[derive(Parser)]
#[clap(name = "convmd")]
struct Cli {
    indir: PathBuf,
    outdir: PathBuf,
    inmat: Mat,
    outmat: Mat,
}

impl FromStr for Mat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self> {
        Ok(match s.to_owned().trim().to_lowercase().as_str() {
            "default" | "d" => Mat::Default,
            "jekyll" | "j" => Mat::Jekyll,
            _ => return Err(format!("No Mat matched for {s}")),
        })
    }
}


fn main() -> Result<()> {
    let cli = Cli::parse();

    let mapping: Box<dyn Mapper> = match (&cli.inmat, &cli.outmat) {
        (Mat::Default, Mat::Jekyll) => Box::new(JekyllMapper::Default),
        _ => unimplemented!("{:?} -> {:?}", cli.inmat, cli.outmat),
    };

    mkdirs(&cli.outdir)?;

    // let p = 
    //     PathBuf::from("/home/minghu6/coding/blog/blog-draft/normal-files/BM.md");
    // let p = 
    //     PathBuf::from("/home/minghu6/coding/blog/blog-draft/normal-files/sa_2016.md");
    // mapping.mapping(&p, &cli.outdir)?;

    for ent in syn_walk(cli.indir)?
        .recursive(false)
        .post_include_ext(&[".md", ".markdown"])
    {
        let ent = ent?;

        mapping.mapping(&ent.path(), &cli.outdir)?;
    }

    Ok(())
}
