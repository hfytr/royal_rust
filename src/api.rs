use reqwest::blocking::{Client, Response};
use select::{
    document::Document,
    node::{Data, Node},
    predicate::{Class, Name},
};

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
    pub content: String,
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
        let content = Self::join_content(document.find(Class("chapter-content")).next()?);
        let chapter = Chapter {
            name: reference.name.to_string(),
            path: reference.path.to_string(),
            content,
            published,
            edited,
        };
        Some(chapter)
    }

    fn join_content(node: Node) -> String {
        match node.data() {
            Data::Element(..) => node
                .children()
                .map(Self::join_content)
                .collect::<Vec<String>>()
                .join("\n"),
            Data::Text(..) => {
                if node.text().as_str() == "\n" {
                    String::new()
                } else {
                    node.text()
                }
            }
            // idk wtf a comment is supposed to mean
            Data::Comment(..) => String::new(),
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
        Some(Fiction { title, chapters })
    }

    pub fn get(&self, path: &str) -> Result<Response, reqwest::Error> {
        self.client
            .get(format!("https://royalroad.com{}", path))
            .send()
    }
}
