pub mod jekyll;

use crate::aux::{file_name, Result};
use lol_html::{element, rewrite_str, RewriteStrSettings};
use pulldown_cmark::{md::push_md, CowStr, Event, Options, Parser, Tag};
use std::path::{Path, PathBuf};

pub trait Mapper {
    fn mapping(&self, input: &Path, outdir: &Path) -> Result<()>;
}

// pub fn common_mapping(text: &str) -> Result<String> {
//     map_img_ref(text)
// }

pub fn map_img_ref<P: AsRef<Path>>(text: &str, imgdir: P) -> Result<String> {
    let imgdir = imgdir.as_ref().to_owned();

    macro_rules! mapdir {
        ($src:expr) => {
            {
                let p = PathBuf::from($src);
                let name = file_name(&p).unwrap();
        
                let newsrcp = imgdir.join(name);
                newsrcp.to_str().unwrap().to_owned()
            }
        };
    }

    macro_rules! maptag {
        ($tag:ident) => {
            match $tag {
                Tag::Image(_link_type, url, _title) => {
                    Tag::Image(
                        _link_type,
                        CowStr::Boxed(
                            mapdir!(url.into_string())
                            .into_boxed_str()
                        ),
                        _title
                    )
                },
                x => x
            }
        };
    }


    let parser = Parser::new_ext(text, Options::all());

    let parser = parser.map(|event| match event {
        Event::Html(tag) => {
            let handler_img_src = element!("img[src]", |img| {
                let src = img.get_attribute("src").unwrap();

                img.set_attribute("src", &mapdir!(src)).unwrap();

                Ok(())
            });

            Event::Html(CowStr::Boxed(
                rewrite_str(
                    &tag,
                    RewriteStrSettings {
                        element_content_handlers: vec![handler_img_src],
                        ..Default::default()
                    },
                )
                .unwrap()
                .into_boxed_str(),
            ))
        }
        Event::Start(tag) => Event::Start(maptag!(tag)),
        Event::End(tag) => Event::End(maptag!(tag)),
        e => e,
    });

    let mut cache = String::new();
    push_md(parser, &mut cache).unwrap();

    Ok(cache)
}
