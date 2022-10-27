
////////////////////////////////////////////////////////////////////////////////
//// Macro

use std::{rc::Rc, collections::HashSet, ffi::OsString, path::Path, fs::DirEntry};

use crate::{
    or2s,
    aux::{ is_dot_file, Result }
};

/// Map inner error into ErrorCode
#[macro_export]
macro_rules! read_dir_wrapper {
    ($path: expr) => {{
        let path = $path;
        match or2s!(std::fs::read_dir(path)) {
            Ok(iter) => {
                Ok(iter.map(|res| match res {
                    Ok(entry) => Ok(entry),
                    Err(err) => Err(format!("{err:?}")),
                }))
            },
            Err(err) => Err(err)
        }

    }};
}

////////////////////////////////////////////////////////////////////////////////
//// Trait

pub trait Exclude = Fn(&Path) -> bool;

////////////////////////////////////////////////////////////////////////////////
//// Structure

#[derive(Clone)]
pub struct FindOptions<'a> {
    pub pre_exclude_opt: Option<Rc<dyn Fn(&Path) -> bool + 'a>>,
    pub post_include_ext_opt: Option<Rc<HashSet<OsString>>>,
    pub exclude_dot: bool,
    pub recursive: bool
}


#[derive(Default)]
pub struct SynWalk<'a> {
    stack: Vec<Result<DirEntry>>,
    opt: FindOptions<'a>,
}


////////////////////////////////////////////////////////////////////////////////
//// Implementation

impl<'a> FindOptions<'a> {
    pub fn with_pre_exclude<F: Exclude + 'a>(mut self, f: F) -> Self {
        self.pre_exclude_opt = Some(Rc::new(f));
        self
    }

    pub fn with_post_include_ext<S: AsRef<str>>(mut self, includes: &[S]) -> Self {
        let post_include_ext =
            Rc::new(HashSet::from_iter(includes.into_iter().map(|s| {
                let s = s.as_ref();
                if s.starts_with(".") {
                    OsString::from(&s[1..])
                } else {
                    OsString::from(&s[..])
                }
            })));

        self.post_include_ext_opt = Some(post_include_ext);
        self
    }

    pub fn recursive(mut self, enable: bool) -> Self {
        self.recursive = enable;
        self
    }

    pub fn verify<P: AsRef<Path>>(&self, p: P) -> bool {
        let path = p.as_ref();

        if path.is_symlink() {
            // Note: path may be both symlink and dir
            return false;
        }

        if path.is_dir() {
            if !self.recursive {
                return false;
            }

            if let Some(ref exclude) = self.pre_exclude_opt {
                if exclude(path) {
                    return false;
                }
            }
            if self.exclude_dot && is_dot_file(path) {
                return false;
            }

            true
        }
        else {
            debug_assert!(path.is_file());

            if let Some(ref post_include_opt) =
                self.post_include_ext_opt
            {
                if let Some(osstr) = path.extension() {
                    if !post_include_opt.contains(osstr) {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            if self.exclude_dot && is_dot_file(&path) {
                return false;
            }

            true
        }

    }

}

impl<'a> Default for FindOptions<'a> {
    fn default() -> Self {
        Self {
            pre_exclude_opt: Default::default(),
            post_include_ext_opt: Default::default(),
            exclude_dot: true,
            recursive: true
        }
    }
}


impl<'a> SynWalk<'a> {
    pub fn pre_exclude<F: Exclude + 'a>(self, f: F) -> Self {
        Self {
            stack: self.stack,
            opt: self.opt.with_pre_exclude(f),
        }
    }

    pub fn post_include_ext<S: AsRef<str>>(self, includes: &[S]) -> Self {
        Self {
            stack: self.stack,
            opt: self.opt.with_post_include_ext(includes),
        }
    }

    pub fn recursive(self, enable: bool) -> Self {
        Self {
            stack: self.stack,
            opt: self.opt.recursive(enable),
        }
    }

    pub fn with_opt(self, opt: FindOptions<'a>) -> Self {
        Self {
            stack: self.stack,
            opt
        }
    }
}

impl<'a> Iterator for SynWalk<'a> {
    type Item = Result<DirEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(res_entry) = self.stack.pop() {
            if let Ok(entry) = res_entry {
                let path = entry.path();

                if self.opt.verify(&path) {
                    if path.is_dir() {
                        match read_dir_wrapper!(path) {
                            Ok(iter) => {
                                self.stack.extend(iter);
                            }
                            Err(err) => {
                                return Some(Err(err));
                            }
                        }
                    }
                    else {
                        return Some(Ok(entry));
                    }

                }
            } else {
                // Some(Err)
                return Some(res_entry);
            }
        }
        None
    }
}



pub fn syn_walk<'a, P: AsRef<Path>>(startdir: P) -> Result<SynWalk<'a>> {
    Ok(SynWalk {
        stack: Vec::from_iter(read_dir_wrapper!(startdir)?),
        opt: FindOptions::default(),
    })
}
