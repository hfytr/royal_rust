use chrono::NaiveDateTime;
use reqwest::blocking::{Client, Response};
use select::{
    document::Document,
    node::{Data, Node},
    predicate::{Child, Class, Name},
};
use serde::{Deserialize, Serialize};
use std::fs::{create_dir_all, read_to_string, File};
use std::io::Write;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct Fiction {
    pub title: String,
    pub id: usize,
    pub chapters: Vec<ChapterReference>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ChapterReference {
    pub path: String,
    pub title: String,
    pub time: u64,
}

#[derive(Debug, Default)]
pub struct Chapter {
    pub name: String,
    pub path: String,
    pub content: Vec<String>,
    pub published: u64,
    pub edited: u64,
}

impl Fiction {
    pub fn write_to_file(path: &str, fictions: &Vec<Fiction>) -> std::io::Result<()> {
        if let Some(parent) = Path::new(path).parent() {
            create_dir_all(parent)?;
        }
        let mut file = File::options()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?;
        file.write_all(
            fictions
                .iter()
                .map(|f| f.id.to_string())
                .collect::<Vec<_>>()
                .join("\n")
                .as_bytes(),
        )?;
        Ok(())
    }

    pub fn from_file(client: &RoyalClient, path: &str) -> std::io::Result<Vec<Fiction>> {
        Ok(read_to_string(path)?
            .split('\n')
            .filter_map(|s| client.get_fiction(s.parse::<usize>().ok()?))
            .collect())
    }
}

fn traverse<'a, 'b>(n: &'a Node, v: &'b Vec<usize>) -> Option<Node<'a>> {
    let mut v = v.iter();
    let mut cur: Node = n.children().nth(*v.next()?)?;
    for i in v {
        cur = cur.children().nth(*i)?;
    }
    Some(cur)
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
struct OfficialChapterReference {
    id: usize,
    volumeId: Option<String>,
    title: String,
    slug: String,
    date: String,
    order: usize,
    visible: usize,
    subscriptionTiers: Option<String>,
    doesNotRollOver: bool,
    isUnlocked: bool,
    url: String,
}

impl From<OfficialChapterReference> for ChapterReference {
    fn from(value: OfficialChapterReference) -> Self {
        Self {
            path: value.url,
            title: value.title,
            time: NaiveDateTime::parse_from_str(&value.date, "%Y-%m-%dT%H:%M:%SZ")
                .unwrap()
                .and_utc()
                .timestamp() as u64,
        }
    }
}

impl Chapter {
    pub fn from_reference(reference: &ChapterReference, client: &RoyalClient) -> Option<Chapter> {
        let result = client.get(&reference.path).ok()?;
        let document = Document::from_read(result.text().ok()?.as_bytes()).ok()?;
        let profile_info: Node = document.find(Class("profile-info")).next().unwrap();
        let published = traverse(&profile_info, &vec![3, 1, 3])?
            .attr("unixtime")?
            .parse::<u64>()
            .ok()?;
        let edited = match traverse(&profile_info, &vec![3, 3, 3]) {
            Some(x) => x.attr("unixtime")?.parse::<u64>().ok()?,
            None => published,
        };
        let mut content = Vec::new();
        Self::join_content(
            document.find(Class("chapter-content")).next()?,
            &mut content,
        );
        let chapter = Chapter {
            name: reference.title.to_string(),
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
                if node.text().as_str() == "\n"
                    && !content.is_empty()
                    && content.last().unwrap().len() != 0
                {
                    content.push(String::new());
                } else if node.text() != "\n" {
                    if content.is_empty() {
                        content.push(String::new());
                    }
                    content.last_mut().unwrap().push_str(&node.text())
                }
            }
            // idk wtf a comment is supposed to mean
            Data::Comment(..) => {}
        }
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

        let possible_chap_lists = document
            .find(Child(Class("page-container-bg-solid"), Name("script")))
            .into_iter()
            .collect::<Vec<_>>();

        let text = possible_chap_lists[possible_chap_lists.len() - 3]
            .children()
            .next()
            .unwrap()
            .text();

        let start_index = text
            .find("window.chapters = ")
            .expect("failed to find chapters")
            + "window.chapters = ".len();
        let skip_initial = &text.as_bytes()[start_index..];

        let mut chapters_json = "";
        let mut stack = 0;
        for (i, byte) in skip_initial.iter().enumerate() {
            if *byte == b'[' {
                stack += 1;
            } else if *byte == b']' {
                stack -= 1;
            }
            if stack == 0 {
                chapters_json = std::str::from_utf8(&skip_initial[0..=i])
                    .expect("failed to convert from json to &str");
                break;
            }
        }

        let chapters = serde_json::from_str::<Vec<OfficialChapterReference>>(chapters_json)
            .expect("failed to parse chapters json")
            .into_iter()
            .map(ChapterReference::from)
            .collect::<Vec<ChapterReference>>();

        Some(Fiction {
            id,
            title,
            chapters,
        })
    }

    pub fn get(&self, path: &str) -> Result<Response, reqwest::Error> {
        self.client
            .get(format!("https://royalroad.com{}", path))
            .send()
    }
}
