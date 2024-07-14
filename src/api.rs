use reqwest::blocking::{Client, Response};
use select::{
    document::Document,
    node::{Data, Node},
    predicate::{Class, Name},
};

use std::time::{SystemTime, UNIX_EPOCH};

pub struct Fiction {
    pub title: String,
    pub chapters: Vec<ChapterReference>,
}

#[derive(Debug, Default)]
pub struct ChapterReference {
    pub path: String,
    pub name: String,
    pub time: u32,
}

#[derive(Debug, Default)]
pub struct Chapter {
    pub name: String,
    pub path: String,
    pub content: Vec<String>,
    pub published: u32,
    pub edited: u32,
}

fn traverse<'a, 'b>(n: &'a Node, v: &'b Vec<usize>) -> Option<Node<'a>> {
    let mut v = v.iter();
    let mut cur: Node = n.children().nth(*v.next()?)?;
    for i in v {
        cur = cur.children().nth(*i)?;
    }
    Some(cur)
}

impl Chapter {
    pub fn from_reference(reference: &ChapterReference, client: &RoyalClient) -> Option<Chapter> {
        let result = client.get(&reference.path).ok()?;
        let document = Document::from_read(result.text().ok()?.as_bytes()).ok()?;
        let profile_info: Node = document.find(Class("profile-info")).next().unwrap();
        let published = traverse(&profile_info, &vec![3, 1, 3])?
            .attr("unixtime")?
            .parse::<u32>()
            .ok()?;
        let edited = match traverse(&profile_info, &vec![3, 3, 3]) {
            Some(x) => x.attr("unixtime")?.parse::<u32>().ok()?,
            None => published,
        };
        let mut content = Vec::new();
        Self::join_content(
            document.find(Class("chapter-content")).next()?,
            &mut content,
        );
        let chapter = Chapter {
            name: reference.name.to_string(),
            path: reference.path.to_string(),
            content,
            published,
            edited,
        };
        Some(chapter)
    }

    fn join_content(node: Node, content: &mut Vec<String>) {
        match node.data() {
            Data::Element(..) => {
                for child in node.children() {
                    Self::join_content(child, content);
                }
            }
            Data::Text(..) => {
                if node.text().as_str() != "\n" {
                    content.push(node.text())
                }
            }
            // idk wtf a comment is supposed to mean
            Data::Comment(..) => {}
        }
    }
}

const MINUTE: u32 = 60;
const HOUR: u32 = 60 * MINUTE;
const DAY: u32 = 24 * HOUR;
const WEEK: u32 = 7 * DAY;
const MONTH: u32 = 30 * DAY;
const YEAR: u32 = 365 * DAY;

impl ChapterReference {
    pub fn from_fiction_page_row(row: &Node) -> Option<ChapterReference> {
        Some(ChapterReference {
            path: row.attr("data-url")?.to_string(),
            time: row
                .find(Name("time"))
                .next()?
                .attr("unixtime")?
                .parse::<u32>()
                .ok()?,
            name: traverse(row, &vec![1, 1, 0])?.text().trim().to_string(),
        })
    }

    pub fn to_string(&self, width: u16, x_margin: u16) -> String {
        let width = width - x_margin * 2;
        let s = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32
            - self.time;
        let mut time = match s {
            YEAR.. => format!("{} years ago", s / YEAR),
            MONTH.. => format!("{} months ago", s / MONTH),
            WEEK.. => format!("{} weeks ago", s / WEEK),
            DAY.. => format!("{} days ago", s / DAY),
            HOUR.. => format!("{} hours ago", s / HOUR),
            MINUTE.. => format!("{} minutes ago", s / MINUTE),
            _ => format!("{} seconds ago", s),
        };
        let mut full_len = self.name.len() + 2 + time.len();
        if full_len > width as usize {
            let words: Vec<&str> = time.split(' ').collect();
            time = format!("{} {}", words[0], words[1]);
            full_len -= 4;
        }
        let spacing_width = (width as i32 - full_len as i32).max(2);
        let spacing = String::from_utf8(vec![b' '; spacing_width as usize]).unwrap();
        let name = self
            .name
            .chars()
            .take(width as usize - 2 * x_margin as usize - time.len() - spacing_width as usize)
            .collect::<String>();
        format!("{}{}{}", name, spacing, time)
    }
}

pub struct RoyalClient {
    client: Client,
}

impl RoyalClient {
    pub fn new() -> RoyalClient {
        RoyalClient {
            client: Client::new(),
        }
    }

    pub fn get_fiction(&self, id: usize) -> Option<Fiction> {
        let full_path = format!("/fiction/{}", id);
        let result = self.get(&full_path).ok()?;
        let document = Document::from_read(result.text().ok()?.as_bytes()).ok()?;
        let title = document.find(Name("h1")).into_iter().next().unwrap().text();
        let chapters: Vec<ChapterReference> = document
            .find(Class("chapter-row"))
            .into_iter()
            .filter_map(|x| ChapterReference::from_fiction_page_row(&x))
            .collect();
        Some(Fiction { title, chapters })
    }

    pub fn get(&self, path: &str) -> Result<Response, reqwest::Error> {
        self.client
            .get(format!("https://royalroad.com{}", path))
            .send()
    }
}
