use std::path::Component;

use pandoc::Pandoc;

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
                Value::Options { values } => {
                    // Create a hidden input variable to store the selected value
                    dom.push(DomElement::Element {
                        tag: "input".to_string(),
                        attributes: vec![
                            ("type".to_string(), "hidden".to_string()),
                            ("id".to_string(), format!("{}", statement.variable)),
                        ],
                        children: Dom(vec![]),
                    });

                    // Create radio buttons for each value
                    for value in values {
                        let event_listener = format!(
                            r#"
    document.getElementById("{}").addEventListener('input', function(event) {{
    document.getElementById("{}").value = "{}";
    document.getElementById("{}").dispatchEvent(new Event('input'));
}});
"#,
                            format!("{}_{}", statement.variable, value),
                            statement.variable,
                            value,
                            statement.variable,
                        );
                        dom.push(DomElement::Element {
                            tag: "input".to_string(),
                            attributes: vec![
                                ("type".to_string(), "radio".to_string()),
                                ("name".to_string(), statement.variable.clone()),
                                ("value".to_string(), value.clone()),
                                (
                                    "id".to_string(),
                                    format!("{}_{}", statement.variable, value),
                                ),
                            ],
                            children: Dom(vec![]),
                        });

                        // Register event listener
                        dom.push(DomElement::Element {
                            tag: "script".to_string(),
                            attributes: vec![],
                            children: Dom(vec![DomElement::Text(event_listener)]),
                        });

                        // Create a label for the radio button

                        dom.push(DomElement::Element {
                            tag: "label".to_string(),
                            attributes: vec![(
                                "for".to_string(),
                                format!("{}_{}", statement.variable, value),
                            )],
                            children: Dom(vec![DomElement::Text(value.clone())]),
                        });
                    }

                    // Create a hidden input variable to store the selected value
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
    Radio,
}

impl ComponentKind {
    pub fn attributes(&self) -> Vec<(String, String)> {
        match self {
            ComponentKind::TextInput => vec![("type".to_string(), "text".to_string())],
            ComponentKind::Radio => vec![("type".to_string(), "radio".to_string())],
            ComponentKind::TextArea | ComponentKind::Paragraph => vec![],
        }
    }

    pub fn accessor(&self) -> String {
        match self {
            ComponentKind::TextInput | ComponentKind::TextArea => "value".to_string(),
            ComponentKind::Paragraph => "innerHTML".to_string(),
            ComponentKind::Radio => "checked".to_string(),
        }
    }

    pub fn tag(&self) -> String {
        match self {
            ComponentKind::TextInput => "input".to_string(),
            ComponentKind::TextArea => "textarea".to_string(),
            ComponentKind::Paragraph => "p".to_string(),
            ComponentKind::Radio => "input".to_string(),
        }
    }
}

#[derive(Clone, Debug)]
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
                    html.push_str(&format!("</{}>", tag));
                }
            }
        }
        html
    }
}

#[derive(Clone, Debug)]
pub enum DomElement {
    Text(String),
    Element {
        tag: String,
        attributes: Vec<(String, String)>,
        children: Dom,
    },
}

impl DomElement {
    pub fn with_attr(&self, key: &str, value: &str) -> Self {
        match self {
            DomElement::Text(text) => DomElement::Text(text.clone()),
            DomElement::Element {
                tag,
                attributes,
                children,
            } =>{
                let mut attributes = attributes.clone();
                if let Some((_, v)) = attributes.iter().find(|(k, _)| k == key) {
                    attributes = attributes
                        .iter()
                        .map(|(k, v)| {
                            if k == key {
                                (k.clone(), value.to_string())
                            } else {
                                (k.clone(), v.clone())
                            }
                        })
                        .collect();
                } else {
                    attributes.push((key.to_string(), value.to_string()));
                }
                
                DomElement::Element {
                tag: tag.clone(),
                attributes: attributes
                    .iter()
                    .map(|(k, v)| {
                        if k == key {
                            (k.clone(), value.to_string())
                        } else {
                            (k.clone(), v.clone())
                        }
                    })
                    .collect(),
                children: children.clone(),
            }},
        }
    }
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
    Options {
        values: Vec<String>,
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
                    "radio" => ComponentKind::Radio,
                    _ => panic!(),
                };
                let pair = pairs.next().unwrap();
                let value = match pair.as_rule() {
                    Rule::constant => {
                        let value = pair.as_str().to_string();
                        Value::Const { value }
                    }
                    Rule::function => {
                        let mut pairs = pair.into_inner();
                        let params = pairs.next().unwrap();
                        let variables = params
                            .as_str()
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .collect();
                        let body = pairs.next().unwrap().as_str().to_string();
                        Value::Fn { variables, body }
                    }
                    Rule::options => {
                        let pairs = pair.into_inner();
                        let values = pairs.map(|p| p.as_str().to_string()).collect();
                        Value::Options { values }
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



pub fn code_block_filter(pandoc: &mut Pandoc) {
    pandoc.add_filter(|json| {
        pandoc_ast::filter(json, |mut pandoc| {
            for block in &mut pandoc.blocks {
                use pandoc_ast::Block::*;
                *block = match block {
                    CodeBlock((ref identifier, ref kinds, ref kvs), ref code) => {
                        let mut dom = vec![];
                        // Create a hidden input variable to store the code as its value
                        dom.push(DomElement::Element {
                            tag: "input".to_string(),
                            attributes: vec![
                                ("type".to_string(), "hidden".to_string()),
                                ("id".to_string(), identifier.clone()),
                                ("value".to_string(), code.replace("\"", "&quot;")),
                            ],
                            children: Dom(vec![]),
                        });

                        if kinds.contains(&"script".to_string()) {
                            
                            dom.push(DomElement::Element {
                                tag: "script".to_string(),
                                attributes: vec![],
                                children: Dom(vec![DomElement::Text(code.clone())]),
                            });

                            if kinds.contains(&"show".to_string()) {

                                if kinds.contains(&"copy".to_string()) {
                                    dom.push(DomElement::Element {
                                        tag: "button".to_string(),
                                        attributes: vec![("onclick".to_string(), format!("navigator.clipboard.writeText(document.getElementById('{}').value);", identifier.clone()))],
                                        children: Dom(vec![DomElement::Text("Copy".to_string())]),
                                    });
                                }

                                let lang = (
                                    "class".to_string(),
                                    format!("language-{}", kinds.first().unwrap().clone()),
                                );
                                let name = ("name".to_string(), identifier.clone());
                                let code = code.trim().replace("<", "&lt;").replace(">", "&gt;");
                                let pre = DomElement::Element {
                                    tag: "pre".to_string(),
                                    attributes: vec![],
                                    children: Dom(vec![DomElement::Element {
                                        tag: "code".to_string(),
                                        attributes: vec![lang, name],
                                        children: Dom(vec![DomElement::Text(code.clone())]),
                                    }]),
                                };

                                if kinds.contains(&"linenumbers".to_string()) {
                                    let number_of_lines = code.lines().count();
                                    // Create a vertical list of numbers
                                    let numbers = (1..=number_of_lines)
                                        .map(|n| format!("<span>{}</span>", n))
                                        .collect::<Vec<String>>()
                                        .join("\n");
                                    let numbers = DomElement::Element {
                                        tag: "pre".to_string(),
                                        attributes: vec![],
                                        children: Dom(vec![DomElement::Element {
                                            tag: "code".to_string(),
                                            attributes: vec![],
                                            children: Dom(vec![DomElement::Text(numbers)]),
                                        }]),
                                    };

                                    let codeblock = DomElement::Element {
                                        tag: "div".to_string(),
                                        attributes: vec![("style".to_string(), "display: flex; flex-direction: row;".to_string())],
                                        children: Dom(vec![numbers, pre.with_attr("style", "flex:1")]),
                                    };

                                    dom.push(codeblock);
                                } else {
                                    dom.push(pre);
                                }
                                
                            }

                            RawBlock(
                                pandoc_ast::Format("HTML".to_string()),
                                Dom(dom).to_raw_html(),
                            )
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
}

fn utf8_meta_filter(pandoc: &mut Pandoc) {
    pandoc.add_filter(|json| {
        pandoc_ast::filter(json, |mut pandoc| {
            let meta_block = DomElement::Element {
                tag: "meta".to_string(),
                attributes: vec![
                    ("charset".to_string(), "UTF-8".to_string()),
                ],
                children: Dom(vec![]),
            };

            pandoc.blocks.insert(0, pandoc_ast::Block::RawBlock(pandoc_ast::Format("HTML".to_string()), Dom(vec![meta_block]).to_raw_html()));

            pandoc
        })
    });
}

fn main() {
    let mut pandoc = pandoc::new();

    code_block_filter(&mut pandoc);    
    utf8_meta_filter(&mut pandoc);

    pandoc.set_input(pandoc::InputKind::Files(vec!["huffman.md"
        .to_string()
        .into()]));

    pandoc.set_output(pandoc::OutputKind::File("huffman.html".into()));
    pandoc.execute().unwrap();
}
