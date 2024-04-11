use pandoc::Pandoc;

mod dcl;
mod dom;

use dcl::interpret_dcl;
use dom::{Dom, DomElement};

fn copy_button(id: &str) -> DomElement {
    DomElement::Element {
        tag: "button".to_string(),
        attributes: vec![(
            "onclick".to_string(),
            format!(
                "navigator.clipboard.writeText(document.getElementById('{}').value);",
                id
            ),
        )],
        children: Dom(vec![DomElement::Text("Copy".to_string())]),
    }
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
                                ("value".to_string(), code.replace('\"', "&quot;")),
                            ],
                            children: Dom(vec![]),
                        });

                        let language = kinds.first().unwrap();

                        if kinds.contains(&"copy".to_string()) {
                            dom.push(copy_button(identifier));
                        }

                        match language.as_str() {
                            "dcl" => {
                                let mut dcl = interpret_dcl(code);
                                dom.append(&mut dcl.0);

                                RawBlock(
                                    pandoc_ast::Format("HTML".to_string()),
                                    Dom(dom).to_raw_html(),
                                )
                            }
                            "mermaid" => {
                                // <script src="https://cdn.jsdelivr.net/npm/mermaid@8/dist/mermaid.min.js"></script>

                                let frame_rate = if let Some((_, value)) = kvs.iter().find(|(k, _)| k == "rate") {
                                    value.parse::<f64>().unwrap()
                                } else {
                                    1000.0
                                };

                                dom.push(DomElement::Element {
                                    tag: "script".to_string(),
                                    attributes: vec![("src".to_string(), "https://cdn.jsdelivr.net/npm/mermaid@8/dist/mermaid.min.js".to_string())],
                                    children: Dom(vec![]),
                                });

                                // <script>
                                //     setInterval(() => {
                                //         let value = document.getElementById("diagram").value;
                                //         let frames = value.split("\n");
                                //         let numberOfFrames = frames.length - 1;
                                //         let currentSecond = Math.floor(Date.now() / 1000);
                                //         let currentFrame = (currentSecond % numberOfFrames) + 1;
                                //         let frameContent = frames.slice(0, currentFrame + 1).join("\n");
                                //         mermaid.render(
                                //             "diagram-renderer",
                                //             frameContent,
                                //             (code) => {document.getElementById("diagram-rendered").innerHTML = code}
                                //         )
                                //     }, 1000);
                                // </script>

                                dom.push(DomElement::Element {
                                    tag: "script".to_string(),
                                    attributes: vec![],
                                    children: Dom(vec![DomElement::Text(format!(
                                        r#"
                                        setInterval(() => {{
                                            let value = document.getElementById("{}").value;
                                            let frames = value.split("\n");
                                            let numberOfFrames = frames.length - 1;
                                            let currentSecond = Math.floor(Date.now() / {});
                                            let currentFrame = (currentSecond % numberOfFrames) + 1;
                                            let frameContent = frames.slice(0, currentFrame + 1).join("\n");
                                            mermaid.render(
                                                "{}-renderer",
                                                frameContent,
                                                (code) => {{document.getElementById("{}-rendered").innerHTML = code}}
                                            )
                                        }}, {});
                                        "#,
                                        identifier, frame_rate, identifier, identifier, frame_rate
                                    ))]),
                                });
                                

                                // <div id="diagram-rendered"></div>

                                dom.push(DomElement::Element {
                                    tag: "div".to_string(),
                                    attributes: vec![("id".to_string(), format!("{}-rendered", identifier.clone()))],
                                    children: Dom(vec![]),
                                });
                                

                                
                            
                                RawBlock(pandoc_ast::Format("HTML".to_string()), Dom(dom).to_raw_html())
                            }
                            _ => {
                                if kinds.contains(&"script".to_string()) {
                                    dom.push(DomElement::Element {
                                        tag: "script".to_string(),
                                        attributes: vec![],
                                        children: Dom(vec![DomElement::Text(code.clone())]),
                                    });

                                    if kinds.contains(&"show".to_string()) {
                                        let lang = (
                                            "class".to_string(),
                                            format!("language-{}", kinds.first().unwrap().clone()),
                                        );
                                        let name = ("name".to_string(), identifier.clone());
                                        let code =
                                            code.trim().replace('<', "&lt;").replace('>', "&gt;");
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
                                                attributes: vec![(
                                                    "style".to_string(),
                                                    "display: flex; flex-direction: row;"
                                                        .to_string(),
                                                )],
                                                children: Dom(vec![
                                                    numbers,
                                                    pre.with_attr("style", "flex:1"),
                                                ]),
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
                                } else {
                                    block.clone()
                                }
                            }
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
                attributes: vec![("charset".to_string(), "UTF-8".to_string())],
                children: Dom(vec![]),
            };

            pandoc.blocks.insert(
                0,
                pandoc_ast::Block::RawBlock(
                    pandoc_ast::Format("HTML".to_string()),
                    Dom(vec![meta_block]).to_raw_html(),
                ),
            );

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

    pandoc.set_output(pandoc::OutputKind::File("huffman1.html".into()));
    pandoc.execute().unwrap();
}
