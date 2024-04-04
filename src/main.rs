use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "markdown.pest"]
struct MarkDownParser;

#[derive(Debug)]
enum Markdown {
    Heading(usize, String),
    // List(Vec<Markdown>),
    Paragraph(Paragraph), // Link(String, String),
                          // Image(String, String),
                          // CodeBlock(String),
}

type URL = String;

#[derive(Debug)]
enum RichText {
    Normal(String),
    Bold(Vec<RichText>),
    Italic(Vec<RichText>),
    Code(Vec<RichText>),
    Image(Vec<RichText>, URL),
    Link(Vec<RichText>, URL),
}

impl RichText {
    pub fn to_html(&self) -> String {
        match self {
            RichText::Normal(text) => text.to_string(),
            RichText::Bold(text) => format!(
                "<b>{}</b>",
                text.iter()
                    .map(|rt| rt.to_html())
                    .collect::<Vec<String>>()
                    .join("")
            ),
            RichText::Italic(text) => format!(
                "<i>{}</i>",
                text.iter()
                    .map(|rt| rt.to_html())
                    .collect::<Vec<String>>()
                    .join("")
            ),
            RichText::Code(text) => format!(
                "<code>{}</code>",
                text.iter()
                    .map(|rt| rt.to_html())
                    .collect::<Vec<String>>()
                    .join("")
            ),
            RichText::Image(text, url) => format!(
                "<img src=\"{}\" alt=\"{}\">",
                url,
                text.iter()
                    .map(|rt| rt.to_text())
                    .collect::<Vec<String>>()
                    .join("")
            ),
            RichText::Link(text, url) => format!(
                "<a href=\"{}\">{}</a>",
                url,
                text.iter()
                    .map(|rt| rt.to_html())
                    .collect::<Vec<String>>()
                    .join("")
            ),
        }
    }

    fn to_text(&self) -> String {
        match self {
            RichText::Normal(text) => text.to_string(),
            RichText::Bold(text) => text
                .iter()
                .map(|rt| rt.to_text())
                .collect::<Vec<String>>()
                .join(""),
            RichText::Italic(text) => text
                .iter()
                .map(|rt| rt.to_text())
                .collect::<Vec<String>>()
                .join(""),
            RichText::Code(text) => text
                .iter()
                .map(|rt| rt.to_text())
                .collect::<Vec<String>>()
                .join(""),
            RichText::Image(text, _) => text
                .iter()
                .map(|rt| rt.to_text())
                .collect::<Vec<String>>()
                .join(""),
            RichText::Link(text, _) => text
                .iter()
                .map(|rt| rt.to_text())
                .collect::<Vec<String>>()
                .join(""),
        }
    }
}

#[derive(Debug)]
struct Paragraph(Vec<RichText>);

impl Paragraph {
    pub fn to_html(&self) -> String {
        self.0.iter().map(|rt| rt.to_html()).collect()
    }
}

struct MarkdownDocument(Vec<Markdown>);

impl MarkDownParser {
    fn parse_rich_text(pair: pest::iterators::Pair<Rule>) -> RichText {
        match pair.as_rule() {
            Rule::normal => RichText::Normal(pair.as_str().to_string()),
            Rule::bold => {
                let mut bold = vec![];
                for inner in pair.into_inner() {
                    bold.push(MarkDownParser::parse_rich_text(inner));
                }
                RichText::Bold(bold)
            }
            Rule::italic => {
                let mut italic = vec![];
                for inner in pair.into_inner() {
                    italic.push(MarkDownParser::parse_rich_text(inner));
                }
                RichText::Italic(italic)
            }
            Rule::code => {
                let mut code = vec![];
                for inner in pair.into_inner() {
                    code.push(MarkDownParser::parse_rich_text(inner));
                }
                RichText::Code(code)
            }
            Rule::link => {
                let mut inner = pair.into_inner();
                let text = MarkDownParser::parse_rich_text(inner.next().unwrap().into_inner().next().unwrap());
                let url = inner.next().unwrap().as_str();
                let url = url[1..url.len()-1].to_string();
                RichText::Link(vec![text], url)
            },
            Rule::image => {
                let mut inner = pair.into_inner();
                let text = MarkDownParser::parse_rich_text(inner.next().unwrap().into_inner().next().unwrap());
                let url = inner.next().unwrap().as_str();
                let url = url[1..url.len()-1].to_string();
                RichText::Image(vec![text], url)
            },
            other => {
                panic!("Unexpected rule: {:?}", other);
            }
        }
    }
}
impl Markdown {
    pub fn from_str(s: &str) -> MarkdownDocument {
        let pairs = MarkDownParser::parse(Rule::document, s).unwrap_or_else(|e| panic!("{}", e));

        println!("Length: {}", s.len());
        let mut markdown = vec![];
        for pair in pairs {
            println!("{:?}", pair);
            match pair.as_rule() {
                Rule::heading => {
                    let mut inner = pair.into_inner();
                    let level = inner.next().unwrap().as_str().len();
                    let text = inner.next().unwrap().as_str().to_string();
                    markdown.push(Markdown::Heading(level, text));
                }
                Rule::paragraph => {
                    let mut paragraph: Vec<RichText> = vec![];
                    for inner in pair.into_inner() {
                        paragraph.push(MarkDownParser::parse_rich_text(inner));
                    }
                    markdown.push(Markdown::Paragraph(Paragraph(paragraph)));
                }
                Rule::empty_line => {}
                Rule::EOI => {}
                
                other => {
                    panic!("Unexpected rule: {:?}", other);
                }
            }
        }

        MarkdownDocument(markdown)
    }

    pub fn to_html(&self) -> String {
        match self {
            Markdown::Heading(level, text) => format!("<h{}>{}</h{}>\n", level, text, level),
            Markdown::Paragraph(text) => format!("<p>{}</p>\n", text.to_html()),
        }
    }
}

impl MarkdownDocument {
    pub fn to_html(&self) -> String {
        self.0.iter().map(|md| md.to_html()).collect()
    }
}

fn main() {
    let md = r#"
# This is a heading 1

This is a paragraph.

This is another paragraph.

# This is another heading 1

## This is a heading 2

This is a paragraph.
Still a paragraph.
Still.

Not anymore.

### This is a heading 3

This is a line.  
This is a paragraph.

This is a **bold** paragraph.

This is a **bold and *italic* ** paragraph.

This is a ***bitalic*** paragraph.

This paragraph has some `code`.

This is a [link](https://example.com).

This is an ![image](https://example.com/image.jpg).

This is a [**bold link**](https://example.com).


"#;

    let doc = Markdown::from_str(md);
    println!("{}", doc.to_html());

    let link = "[link]";
    let pairs = MarkDownParser::parse(Rule::link_text, link).unwrap_or_else(|e| panic!("{}", e));

}
