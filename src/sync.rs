pub struct Sync {
    pub fer: ferinth::Ferinth<()>
}

impl Sync {
    pub fn new() -> Self {
        Self {
            fer: ferinth::Ferinth::new(env!("CARGO_PKG_REPOSITORY"), Some(env!("CARGO_PKG_VERSION")), Some(env!("CARGO_PKG_HOMEPAGE")))
        }
    }
}
