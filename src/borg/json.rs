#[derive(Serialize, Deserialize, Clone, Debug, Hash, Ord, Eq, PartialOrd, PartialEq)]
pub struct RepoId(String);

impl RepoId {
    pub fn new(id: String) -> Self {
        Self(id)
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, Ord, Eq, PartialOrd, PartialEq)]
pub struct ArchiveId(String);

impl ArchiveId {
    pub fn new(id: String) -> Self {
        Self(id)
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, Ord, Eq, PartialOrd, PartialEq)]
pub struct ArchiveName(String);

impl ArchiveName {
    pub fn new(id: String) -> Self {
        Self(id)
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Stats {
    pub archive: NewArchive,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NewArchive {
    pub duration: f64,
    pub id: ArchiveId,
    pub name: ArchiveName,
    pub stats: NewArchiveSize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NewArchiveSize {
    pub compressed_size: u64,
    pub deduplicated_size: u64,
    pub nfiles: u64,
    pub original_size: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct List {
    pub archives: Vec<Archive>,
    pub encryption: Encryption,
    pub repository: Repository,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Archive {
    pub id: ArchiveId,
    pub name: ArchiveName,
    pub comment: String,
    pub username: String,
    pub hostname: String,
    pub start: chrono::naive::NaiveDateTime,
    pub end: chrono::naive::NaiveDateTime,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Encryption {
    pub mode: String,
    pub keyfile: Option<std::path::PathBuf>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Repository {
    pub id: RepoId,
    pub last_modified: chrono::naive::NaiveDateTime,
    pub location: std::path::PathBuf,
}