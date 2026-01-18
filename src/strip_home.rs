use std::{env::home_dir, path::Path};

pub fn path_to_human_readable(path: &Path) -> String {
    let home_stripped = strip_home(path);

    let stringified_path = match home_stripped {
        Some(home_stripped) => home_stripped.to_str(),
        None => path.to_str(),
    };

    if let Some(stringified_path) = stringified_path
        && home_stripped.is_some()
    {
        format!("~/{stringified_path}")
    } else {
        stringified_path
            .unwrap_or("Kann Dateipfad nicht darstellen")
            .to_string()
    }
}

fn strip_home(path: &Path) -> Option<&Path> {
    path.strip_prefix(home_dir()?).ok()
}
