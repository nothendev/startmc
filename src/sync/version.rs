use std::path::Path;

#[derive(Debug)]
pub struct VersionTuple {
    pub name: String,
    pub version: String,
}

impl VersionTuple {
    pub fn new(name: String, version: String) -> Self {
        Self { name, version }
    }

    pub fn parse(input: &str) -> Option<Self> {
        if !input.contains('-') {
            let path = Path::new(input);
            return Some(Self {
                name: path
                    .file_stem()
                    .map(|it| it.to_str().unwrap().to_string())
                    .unwrap_or_else(|| input.to_string()),
                version: "0.0.0".to_string(),
            });
        }

        let mut parts = input
            .trim_end_matches(".jar")
            .trim_end_matches(".zip")
            .split('-');
        let mut name = String::with_capacity(input.len() / 2 + 4);

        while let Some(part) = parts.next() {
            if part.chars().next().unwrap().is_ascii_digit() {
                return Some(Self {
                    name,
                    version: [part]
                        .into_iter()
                        .chain(parts)
                        .take_while(|it| !it.contains(".jar") && !it.contains(".zip"))
                        .collect::<Vec<_>>()
                        .join("-"),
                });
            } else {
                name.push_str(part);
            }
        }

        None
    }

    pub fn parse_eq(input: &str) -> Option<Self> {
        if !input.contains('=') {
            return Some(Self {
                name: input.to_string(),
                version: "".to_string(),
            });
        }
        let mut parts = input.split('=');
        let name = parts.next()?.to_string();
        let version = parts.next()?.to_string();
        Some(Self { name, version })
    }
}
