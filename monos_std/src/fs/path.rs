use alloc::string::{String, ToString};

#[derive(Debug)]
pub struct Path<'p>(&'p str);
#[derive(Debug, Clone)]
pub struct ArrayPath {
    length: u8,
    data: [u8; 255],
}
#[derive(Debug, Clone)]
pub struct PathBuf(String);

impl<'p> Path<'p> {
    pub const fn new(path: &'p str) -> Self {
        Self(path)
    }

    pub const fn as_str(&self) -> &str {
        self.0
    }

    pub fn file_name(&self) -> Option<&str> {
        self.0.rsplit('/').next()
    }

    pub fn extension(&self) -> Option<&str> {
        let file_name = self.file_name()?;
        let dot_index = file_name.rfind('.')?;
        Some(&file_name[dot_index + 1..])
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

impl<'p> From<&'p ArrayPath> for Path<'p> {
    #[inline]
    fn from(path: &'p ArrayPath) -> Path<'p> {
        Path(path.as_str())
    }
}

impl From<ArrayPath> for PathBuf {
    #[inline]
    fn from(path: ArrayPath) -> PathBuf {
        PathBuf(path.into())
    }
}

impl core::fmt::Display for Path<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ArrayPath {
    pub fn new() -> Self {
        Self {
            length: 0,
            data: [0; 255],
        }
    }

    pub fn push_str(&mut self, path: &str) {
        let new_length = self.length.saturating_add(path.len() as u8);
        self.data[self.length as usize..new_length as usize].copy_from_slice(path.as_bytes());
        self.length = new_length
    }

    pub fn as_str(&self) -> &str {
        let length = self.length as usize;

        // safety: we know this is valid utf8 because we only write valid utf8 to the buffer
        unsafe { core::str::from_utf8_unchecked(&self.data[..length]) }
    }
}

impl From<&str> for ArrayPath {
    fn from(path: &str) -> ArrayPath {
        let mut data = [0; 255];
        let length = path.len().min(255);
        data[..length].copy_from_slice(path.as_bytes());
        ArrayPath {
            length: length as u8,
            data,
        }
    }
}

impl From<String> for ArrayPath {
    fn from(path: String) -> ArrayPath {
        path.as_str().into()
    }
}

impl From<ArrayPath> for String {
    fn from(path: ArrayPath) -> String {
        path.as_str().to_string()
    }
}

impl From<PathBuf> for ArrayPath {
    fn from(path: PathBuf) -> ArrayPath {
        path.0.into()
    }
}

impl From<&Path<'_>> for ArrayPath {
    fn from(path: &Path) -> ArrayPath {
        path.0.into()
    }
}

impl PathBuf {
    pub fn new() -> Self {
        Self(String::new())
    }

    pub fn from_str(path: &str) -> Self {
        Self(path.to_string())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
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

impl From<Path<'_>> for PathBuf {
    #[inline]
    fn from(path: Path) -> PathBuf {
        PathBuf(path.0.to_string())
    }
}

impl From<String> for PathBuf {
    #[inline]
    fn from(path: String) -> PathBuf {
        PathBuf(path)
    }
}

impl From<&str> for PathBuf {
    #[inline]
    fn from(path: &str) -> PathBuf {
        PathBuf(path.to_string())
    }
}

impl From<PathBuf> for String {
    #[inline]
    fn from(path: PathBuf) -> String {
        path.0
    }
}
