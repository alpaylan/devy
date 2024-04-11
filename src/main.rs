use pandoc::Pandoc;

mod dcl;
mod dom;

use dcl::interpret_dcl;
use dom::{Dom, DomElement};

pub fn code_block_filter(pandoc: &mut Pandoc) {
    pandoc.add_filter(|json| {
        pandoc_ast::filter(json, |mut pandoc| {
            for block in &mut pandoc.blocks {
                use pandoc_ast::Block::*;
                *block = match block {
                    CodeBlock((ref identifier, ref kinds, ref _kvs), ref code) => {
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
                                let code = code.trim().replace('<', "&lt;").replace('>', "&gt;");
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

    pandoc.set_output(pandoc::OutputKind::File("huffman.html".into()));
    pandoc.execute().unwrap();
}
