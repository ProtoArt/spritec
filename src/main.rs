use std::fs::File;
use std::io::Error as IOError;
use fbx_direct::reader::{EventReader, FbxEvent::*, Error as FBXError};

// TODO: Pick a better name than AppError
#[derive(Debug)]
enum AppError {
    IOError(IOError),
    FBXError(FBXError),
}

impl From<IOError> for AppError {
    fn from(error: IOError) -> Self {
        AppError::IOError(error)
    }
}

impl From<FBXError> for AppError {
    fn from(error: FBXError) -> Self {
        AppError::FBXError(error)
    }
}

fn main() -> Result<(), AppError> {
    let file = File::open("samples/simple.fbx")?;
    let reader = EventReader::new(file);

    for event_res in reader {
        match event_res? {
            StartFbx(_) => {},
            EndFbx => {},
            StartNode {name, properties} => {
                println!("name: {:?}, properties: {:?}", name, properties);
            },
            EndNode => println!("End Node"),
            Comment(_) => {},
        }
    }

    Ok(())
}
