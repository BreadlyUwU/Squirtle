use std::{fs, io::{Error, ErrorKind}};

/// Read a value inside the SysFS
pub fn sniffa<P: AsRef<std::path::Path>>(location: P) -> std::io::Result<String> {
    let location = location.as_ref();

    match fs::read_to_string(location) {
        Ok(content) => { 
            log::debug!("Reading {:?}", &location);
            return Ok(content.trim().to_string()) 
        },
        Err(e) => {
            log::error!("{ERR_READ_MSG} {:?}:{:?}", &location, e);
            Err(e) 
        }
    }
}

/// Write a new value at a specific SysFS path
pub fn boop<P: AsRef<std::path::Path>, V: ToString>(location: P, value: V) -> std::io::Result<()> {
    let location = location.as_ref();
    match fs::exists(location) {
        Ok(true) => match fs::write(location, value.to_string()) {
            Ok(_) => {
                log::debug!("Wrote \"{}\" at {:?}", &value.to_string(), &location);
                Ok(())
            },
            Err(e) => {
                log::error!("{ERR_WRITE_MSG} {:?}: {e:?}", location);
                Err(e)
            },
        },
        Ok(false) => {
            let error = Err(Error::new(ErrorKind::NotFound, format!("Tried to write a value inside a SysFS path that doesn't exists: {}", location.display())));
            log::error!("{ERR_WRITE_MSG} {:?}", &error);
            return error
        },
        Err(e) => { eprintln!("Unexpected error: {}", e); Err(e) },
    }
}

const ERR_READ_MSG: &str = "Error while trying to read a SysFS value:";
const ERR_WRITE_MSG: &str = "Error while trying to write inside SysFS:";