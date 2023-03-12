use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use bytes::Bytes;
use lazy_static::lazy_static;
use regex::Regex;

pub struct Paragraph {
    pub original_text: String,
    pub cleaned_text: String,
    pub audio: Option<Bytes>,
}


impl Paragraph {
    pub fn new(original_text: String) -> Self {
        Paragraph { original_text, cleaned_text: String::new(), audio: None }
    }

    fn clean_text(&mut self) {
        for line in self.original_text.split('\n') {
            let cleaned = remove_all_dash(&remove_image_link(&remove_url(line)));
            if !cleaned.is_empty() {
                self.cleaned_text.push_str(&cleaned);
                if !cleaned.ends_with(',') && !cleaned.ends_with('.') && !cleaned.ends_with('?') {
                    self.cleaned_text.push(',');
                }
            }
        }
        if self.cleaned_text.ends_with(',') {
            self.cleaned_text.pop();
        }
    }
}

impl From<String> for Paragraph {
    fn from(value: String) -> Self {
        Paragraph::new(value)
    }
}


fn remove_all_dash(s: &str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new("^-+?$").unwrap();
    }
    RE.replace(s, "").to_string()
}

fn remove_image_link(s: &str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r#"!\[[^]]*]"#).unwrap();
    }
    RE.replace_all(s, "").to_string()
}

fn remove_url(s: &str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r#"\(http[^)]*\)"#).unwrap();
    }
    RE.replace_all(s, "").to_string()
}

/// divide note content by blank line to paragraphs
/// including file name
pub fn divide_note_content(path: &Path) -> Vec<Paragraph> {
    let mut paragraphs: Vec<Paragraph> = vec![];
    if let Some(s) = path.file_stem() {
        paragraphs.push(s.to_str().unwrap().to_string().into())
    }
    let reader = BufReader::new(File::open(path).unwrap());
    let mut p = String::new();
    for line in reader.lines().flatten() {
        if line.is_empty() && !p.is_empty() {
            paragraphs.push(p.clone().into());
            p.clear();
        } else {
            p.push_str(&line);
            p.push('\n');
        }
    }
    if !p.is_empty() {
        paragraphs.push(p.into());
    }

    paragraphs.iter_mut().for_each(|p| p.clean_text());
    paragraphs
}

#[cfg(test)]
mod tests {
    use super::{remove_all_dash, remove_image_link, remove_url};

    #[test]
    fn remove_all_dash_test() {
        let line = "----";
        assert!(remove_all_dash(line).is_empty());
    }

    #[test]
    fn remove_url_test() {
        let line = "[abc](https://foo.bar)ok)";
        assert_eq!("[abc]ok)", remove_url(line));
    }

    #[test]
    fn remove_image_link_test() {
        let line = "![|400](https://www.themoviedb.org/t/p/w1280/dqW5ZCgi8R0Rsi72BXAf2AQGzb1.jpg)";
        assert_eq!("(https://www.themoviedb.org/t/p/w1280/dqW5ZCgi8R0Rsi72BXAf2AQGzb1.jpg)", remove_image_link(line));
    }
}
