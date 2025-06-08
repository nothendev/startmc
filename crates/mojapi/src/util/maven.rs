use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MavenVersion {
    pub group: String,
    pub artifact: String,
    pub version: String,
}

impl MavenVersion {
    pub fn parse(s: &str) -> Option<Self> {
        let mut parts = s.split(':');
        let group = parts.next()?.to_string();
        let artifact = parts.next()?.to_string();
        let version = parts.next()?.to_string();

        Some(Self {
            group,
            artifact,
            version,
        })
    }

    pub fn get_filename(&self) -> String {
        format!("{}-{}.jar", self.artifact, self.version)
    }

    pub fn get_path(&self) -> String {
        format!("{group}/{artifact}/{version}/{artifact}-{version}.jar", group = self.group.replace('.', "/"), artifact = self.artifact, version = self.version)
    }

    pub fn get_url(&self, repo: &str) -> String {
        format!("{repo}/{}", self.get_path())
    }
}

impl Display for MavenVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.group, self.artifact, self.version)
    }
}
