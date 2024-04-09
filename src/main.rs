use std::path::Component;

use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "dcl.pest"]
struct DclParser;

fn raw_script(s: &str) -> pandoc_ast::Block {
    pandoc_ast::Block::RawBlock(
        pandoc_ast::Format("HTML".to_string()),
        format!("<script>{}</script>", s),
    )
}

#[derive(Debug)]
pub struct DeclarativeComponentLanguage {
    pub statements: Vec<Statement>,
}

impl DeclarativeComponentLanguage {
    pub fn to_dom(&self) -> Dom {
        let mut dom = vec![];

        for statement in &self.statements {
            match &statement.value {
                Value::Const { value } => dom.push(DomElement::Element {
                    tag: statement.component_kind.tag(),
                    attributes: [
                        statement.component_kind.attributes(),
                        vec![
                            ("id".to_string(), statement.variable.clone()),
                            ("value".to_string(), value.clone()),
                        ],
                    ]
                    .concat(),
                    children: Dom(vec![]),
                }),
                Value::Fn { variables, body } => {
                    // Create event listeners for each variable, and update the value of the input
                    let mut body_with_query_selectors = body.clone();
                    for variable in variables {
                        body_with_query_selectors = body_with_query_selectors.replace(
                            variable,
                            &format!("document.getElementById(\"{}\").value", variable),
                        );
                    }

                    for variable in variables {
                        let event_listener = format!(
                            r#"
    document.getElementById("{}").addEventListener('input', function(event) {{
    document.getElementById("{}").{} = {}
}});
"#,
                            variable,
                            statement.variable,
                            statement.component_kind.accessor(),
                            body_with_query_selectors
                        );
                        dom.push(DomElement::Element {
                            tag: "script".to_string(),
                            attributes: vec![],
                            children: Dom(vec![DomElement::Text(event_listener)]),
                        });
                    }

                    dom.push(DomElement::Element {
                        tag: statement.component_kind.tag(),
                        attributes: [
                            statement.component_kind.attributes(),
                            vec![("id".to_string(), statement.variable.clone())],
                        ]
                        .concat(),
                        children: Dom(vec![]),
                    });
                }
            };
        }

        Dom(dom)
    }
}

#[derive(Debug)]
pub struct Statement {
    pub variable: String,
    pub component_kind: ComponentKind,
    pub value: Value,
}

#[derive(Debug)]
pub enum ComponentKind {
    TextInput,
    TextArea,
    Paragraph,
}

impl ComponentKind {
    pub fn attributes(&self) -> Vec<(String, String)> {
        match self {
            ComponentKind::TextInput => vec![("type".to_string(), "text".to_string())],
            ComponentKind::TextArea | ComponentKind::Paragraph => vec![],
        }
    }

    pub fn accessor(&self) -> String {
        match self {
            ComponentKind::TextInput | ComponentKind::TextArea => "value".to_string(),
            ComponentKind::Paragraph => "innerHTML".to_string(),
        }
    }

    pub fn tag(&self) -> String {
        match self {
            ComponentKind::TextInput => "input".to_string(),
            ComponentKind::TextArea => "textarea".to_string(),
            ComponentKind::Paragraph => "p".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct Dom(pub Vec<DomElement>);

impl Dom {
    pub fn to_raw_html(&self) -> String {
        let mut html = String::new();
        for element in &self.0 {
            match element {
                DomElement::Text(text) => html.push_str(text),
                DomElement::Element {
                    tag,
                    attributes,
                    children,
                } => {
                    html.push_str(&format!("<{}", tag));
                    for (key, value) in attributes {
                        html.push_str(&format!(" {}=\"{}\" ", key, value));
                    }
                    html.push_str(">");
                    html.push_str(&children.to_raw_html());
                    html.push_str(&format!("</{}>\n", tag));
                }
            }
        }
        html
    }
}

#[derive(Debug)]
pub enum DomElement {
    Text(String),
    Element {
        tag: String,
        attributes: Vec<(String, String)>,
        children: Dom,
    },
}

#[derive(Debug)]
pub enum Value {
    Fn {
        variables: Vec<String>,
        body: String,
    },
    Const {
        value: String,
    },
}

fn parse_dcl(s: &str) -> DeclarativeComponentLanguage {
    let pairs = DclParser::parse(Rule::document, s).unwrap_or_else(|e| panic!("{}", e));

    let mut statements = vec![];

    for pair in pairs {
        let statement = match pair.as_rule() {
            Rule::stmt => {
                let mut pairs = pair.into_inner();
                let variable = pairs.next().unwrap().as_str().trim().to_string();
                let component_kind = match pairs.next().unwrap().as_str() {
                    "text-input" => ComponentKind::TextInput,
                    "text-area" => ComponentKind::TextArea,
                    "paragraph" => ComponentKind::Paragraph,
                    _ => panic!(),
                };
                let pair = pairs.next().unwrap();
                let value = match pair.as_rule() {
                    Rule::constant => {
                        let value = pair.as_str().to_string();
                        Value::Const { value }
                    }
                    Rule::function => {
                        println!("{:?}", pair);
                        let mut pairs = pair.into_inner();
                        println!("{:?}", pairs);
                        let params = pairs.next().unwrap();
                        let variables = params
                            .as_str()
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .collect();
                        let body = pairs.next().unwrap().as_str().to_string();
                        Value::Fn { variables, body }
                    }
                    other => panic!("{:?}", other),
                };
                Statement {
                    variable,
                    component_kind,
                    value,
                }
            }
            Rule::EOI => continue,
            other => panic!("{:?}", other),
        };
        statements.push(statement);
    }

    DeclarativeComponentLanguage { statements }
}

fn interpret_dcl(s: &str) -> pandoc_ast::Block {
    let dcl = parse_dcl(s);
    let dom = dcl.to_dom();
    let html = dom.to_raw_html();
    pandoc_ast::Block::RawBlock(pandoc_ast::Format("HTML".to_string()), html)
}

fn main() {
    let mut pandoc = pandoc::new();

    pandoc.add_filter(|json| {
        pandoc_ast::filter(json, |mut pandoc| {
            for block in &mut pandoc.blocks {
                use pandoc_ast::Block::*;
                *block = match block {
                    CodeBlock((ref identifiers, ref kinds, ref kvs), ref code) => {
                        if kinds.contains(&"script".to_string()) {
                            raw_script(code)
                        } else if kinds.contains(&"dcl".to_string()) {
                            interpret_dcl(code)
                        } else {
                            block.clone()
                        }
                    }
                    _ => block.clone(),
                }
            }
            pandoc
        })
    });

    pandoc.set_input(pandoc::InputKind::Files(vec!["huffman.md"
        .to_string()
        .into()]));

    pandoc.set_output(pandoc::OutputKind::File("huffman.html".into()));
    pandoc.execute().unwrap();
}
