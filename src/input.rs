use std::fs::File;
use std::io::prelude::*;


/// Long arguments can be specified with the @FILEPATH to make the content of a file instead
pub fn long_argument(value: &str) -> Result<String, String> {
    if value.starts_with('@') {
        let mut content = String::new();
        let path = &value[1..];
        let mut file = File::open(path).map_err(|err| format!("Could not open file {}: {}", path, err))?;
        file.read_to_string(&mut content).map(|_| content).map_err(|err| format!("Could not read from the file {}: {}", path, err))
    } else {
        Ok(value.to_owned())
    }
}
