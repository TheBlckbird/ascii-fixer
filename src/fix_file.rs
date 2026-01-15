use std::{fs, io, path::Path};

pub enum FixFileError {
    #[allow(unused)]
    Io(io::Error),
    InvalidFilename,
}

impl From<io::Error> for FixFileError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

pub fn fix_file(path: &Path) -> Result<(), FixFileError> {
    if !path.exists() {
        return Err(FixFileError::InvalidFilename);
    }

    if path.is_dir() {
        let dir_contents = fs::read_dir(path)?;

        for dir_entry in dir_contents {
            fix_file(&dir_entry?.path())?;
        }

        return Ok(());
    }

    let file_content = fs::read(path)?;
    let new_string = remove_non_ascii(&file_content);

    fs::write(path, new_string)?;

    Ok(())
}

fn remove_non_ascii(input: &[u8]) -> String {
    let bytes: Vec<u8> = input
        .iter()
        .map(|&character| {
            if (0b00100000..0b01111111).contains(&character)
                || character == b'\n'
                || character == b'\t'
            {
                character
            } else {
                0b00100000
            }
        })
        .collect();

    String::from_utf8(bytes).expect("New String should only contain ASCII data")
}
