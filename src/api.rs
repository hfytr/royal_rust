use reqwest::blocking::{Client, Response};
use select::{
    document::Document,
    node::{Data, Node},
    predicate::{Class, Name},
};

pub struct Fiction {
    title: String,
    chapters: Vec<Chapter>,
}

#[derive(Debug, Default)]
pub struct Chapter {
    name: String,
    path: String,
    content: String,
    published: u32,
    edited: u32,
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
    pub fn from_fiction_page_row(node: Node, client: &RoyalClient) -> Option<Chapter> {
        Self::from_path(
            node.children().nth(1)?.children().nth(1)?.attr("href")?,
            client,
        )
    }

    pub fn from_path(path: &str, client: &RoyalClient) -> Option<Chapter> {
        let result = client.get(path).ok()?;
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
        let fic_header = document.find(Class("fic-header")).next()?;
        let name = traverse(&fic_header, &vec![1, 3, 8, 0])?.text();
        let content = Self::join_content(document.find(Class("chapter-content")).next()?);
        let chapter = Chapter {
            name,
            path: path.to_string(),
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
        let chapters: Vec<Chapter> = document
            .find(Class("chapter-row"))
            .into_iter()
            .filter_map(|x| Chapter::from_fiction_page_row(x, self))
            .collect();
        Some(Fiction { title, chapters })
    }

    pub fn get(&self, path: &str) -> Result<Response, reqwest::Error> {
        self.client
            .get(format!("https://royalroad.com{}", path))
            .send()
    }
}
