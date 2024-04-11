



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


