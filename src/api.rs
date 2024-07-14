use reqwest::blocking::{Client, Response};
use select::{
    document::Document,
    node::{Data, Node},
    predicate::{Class, Name},
};
use std::fs::{create_dir_all, read_to_string, File};
use std::io::Write;
use std::path::Path;

#[derive(Debug)]
pub struct Fiction {
    pub title: String,
    pub id: usize,
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
                .into_iter()
                .map(|f| {
                    format!(
                        "{}.{}.{}-",
                        f.id,
                        f.title,
                        f.chapters
                            .iter()
                            .map(|c| format!("{},{},{}.", c.path, c.name, c.time))
                            .fold(String::new(), |acc, elem| format!("{}{}", acc, elem))
                    )
                })
                .fold(String::new(), |acc, elem| format!("{}{}", acc, elem))
                .as_bytes(),
        )?;
        Ok(())
    }

    pub fn from_file(path: &str) -> std::io::Result<Vec<Fiction>> {
        let file_string = read_to_string(path)?;
        Ok(file_string
            .split('-')
            .map(|s| {
                let mut split = s.split('.');
                Fiction {
                    id: split.next().unwrap().parse::<usize>().unwrap(),
                    title: split.next().unwrap().to_string(),
                    chapters: split
                        .map(|s| {
                            let mut split = s.split(',');
                            ChapterReference {
                                path: split.next().unwrap().to_string(),
                                name: split.next().unwrap().to_string(),
                                time: split.next().unwrap().parse::<u32>().unwrap(),
                            }
                        })
                        .collect::<Vec<ChapterReference>>(),
                }
            })
            .collect::<Vec<Fiction>>())
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
