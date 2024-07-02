use alloc::string::{String, ToString};

#[derive(Debug)]
pub struct Path<'p>(&'p str);
#[derive(Debug)]
pub struct PathBuf(String);

impl<'p> Path<'p> {
    pub fn new(path: &'p str) -> Self {
        Self(path)
    }

    pub fn as_str(&self) -> &str {
        self.0
    }

    pub fn file_name(&self) -> Option<&str> {
        self.0.rsplit('/').next()
    }

    pub fn parent(&self) -> Option<Path<'_>> {
        let parent = self.0.rsplit('/').skip(1).next();
        parent.map(|parent| Path(parent))
    }

    pub fn enter(&self) -> Option<(Path<'_>, Path<'_>)> {
        let mut parts = self.0.splitn(2, '/');
        let first = parts.next()?;
        let second = parts.next()?;
        Some((Path(first), Path(second)))
    }

    pub fn to_path_buf(&self) -> PathBuf {
        PathBuf(self.0.to_string())
    }
}

impl AsRef<str> for Path<'_> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.0
    }
}

impl<'p> From<&'p str> for Path<'p> {
    #[inline]
    fn from(path: &'p str) -> Path<'p> {
        Path(path)
    }
}

impl<'p> From<&'p PathBuf> for Path<'p> {
    #[inline]
    fn from(path: &'p PathBuf) -> Path<'p> {
        Path(path.0.as_str())
    }
}

impl PathBuf {
    pub fn new() -> Self {
        Self(String::new())
    }

    pub fn child(&mut self, path: &str) {
        if !path.starts_with('/') {
            self.0.push('/');
        }
        self.0.push_str(path);
    }
}

impl AsRef<str> for PathBuf {
    #[inline]
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}
