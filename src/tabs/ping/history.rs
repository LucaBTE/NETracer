use std::{fs, io::{self, Read, Write}, path::PathBuf};
use directories::ProjectDirs;
use tempfile::NamedTempFile;
use crate::network::target::Target;

const MAX_HISTORY_BYTES: u64 = 1024 * 1024;

pub(super) struct History {
    path: PathBuf,
}

impl History {
    pub fn discover() -> io::Result<Self> {
        let dirs = ProjectDirs::from("", "", "netracer")
            .ok_or_else(|| io::Error::other("Cannot locate the user state directory"))?;
        // Preserve feature/ping's Linux path and plain-text file format.
        let base = dirs.state_dir().unwrap_or_else(|| dirs.data_local_dir());
        Ok(Self { path: base.join("recent-targets.txt") })
    }

    pub fn load(&self) -> io::Result<Vec<Target>> {
        let file = match fs::File::open(&self.path) {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(error),
        };
        let mut content = String::new();
        file.take(MAX_HISTORY_BYTES + 1).read_to_string(&mut content)?;
        if content.len() as u64 > MAX_HISTORY_BYTES {
            return Err(io::Error::other("History exceeds the 1 MiB limit"));
        }
        let mut targets = Vec::new();
        for line in content.lines() {
            if let Ok(target) = Target::parse(line)
                && !targets.contains(&target)
            {
                targets.push(target);
            }
        }
        Ok(targets)
    }

    pub fn save<'a>(&self, targets: impl Iterator<Item = &'a Target>) -> io::Result<()> {
        let parent = self.path.parent().ok_or_else(|| io::Error::other("Invalid history path"))?;
        fs::create_dir_all(parent)?;
        let mut file = NamedTempFile::new_in(parent)?;
        let mut size = 0;
        for target in targets {
            size += target.as_str().len() as u64 + 1;
            if size > MAX_HISTORY_BYTES {
                return Err(io::Error::other("History exceeds the 1 MiB limit"));
            }
            writeln!(file, "{target}")?;
        }
        file.as_file().sync_all()?;
        file.persist(&self.path).map_err(|error| error.error)?;
        Ok(())
    }

    #[cfg(test)]
    pub fn at(path: PathBuf) -> Self { Self { path } }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_old_format_and_persists_removal() {
        let dir = tempfile::tempdir().unwrap();
        let history = History::at(dir.path().join("recent-targets.txt"));
        fs::write(&history.path, "EXAMPLE.com\nexample.com\nhttps://bad\n::1\n").unwrap();
        let mut targets = history.load().unwrap();
        assert_eq!(targets.len(), 2);
        targets.remove(0);
        history.save(targets.iter()).unwrap();
        assert_eq!(history.load().unwrap(), targets);
        targets.clear();
        history.save(targets.iter()).unwrap();
        assert!(history.load().unwrap().is_empty());
    }
}
