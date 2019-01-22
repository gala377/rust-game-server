/// Helper functions for operations on files.

use std::error::Error;
use std::fs::File;
use std::io::Read;

#[cfg(test)]
use std::io::Write;

/// Reads the whole file under path to the string.
pub fn read(file_name: &str) -> Result<String, Box<Error>> {
    let mut file = File::open(file_name)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    Ok(content)
}

/// Creates temp file with the given content
#[cfg(test)]
pub fn create_temp_with_content(content: &str) -> Result<tempfile::NamedTempFile, Box<Error>> {
    let mut tmp_file = tempfile::NamedTempFile::new()?;
    tmp_file.write_all(content.as_bytes())?;
    Ok(tmp_file)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_existing_file_returns_error() {
        assert!(read("nonexisting_file.toml").is_err());
    }

    #[test]
    fn read_content_is_proper() {
        let content = "Some file content";
        let file = create_temp_with_content(&content).unwrap();
        let file_content = read(file.path().to_str().unwrap()).unwrap();
        assert_eq!(content, file_content);
    }
}
