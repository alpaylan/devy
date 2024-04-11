use pest::Parser;
use pest_derive::Parser;

use crate::dom::{Dom, DomElement};

#[derive(Parser)]
#[grammar = "dcl.pest"]
struct DclParser;

#[derive(Debug)]
pub struct DeclarativeComponentLanguage {
    pub statements: Vec<Statement>,
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
                        dom.push(DomElement::script(&event_listener));
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
                            ("id".to_string(), statement.variable.to_string()),
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
                            format_args!("{}_{}", statement.variable, value),
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

pub fn parse_dcl(s: &str) -> DeclarativeComponentLanguage {
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

pub fn interpret_dcl(s: &str) -> pandoc_ast::Block {
    let dcl = parse_dcl(s);
    let dom = dcl.to_dom();
    let html = dom.to_raw_html();
    pandoc_ast::Block::RawBlock(pandoc_ast::Format("HTML".to_string()), html)
}
