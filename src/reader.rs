use std::io;
use std::fs;
use std::fmt;
use std::result;
use std::error::Error;
use std::path::Path;
use std::convert::From;
use std::string::FromUtf8Error;

use crate::consts::*;

pub struct NCDimension {
    name: String,
    length: u32,
}

pub enum NCAttribute {
    Byte(Vec<u8>),
    Char(Vec<char>),
    Short(Vec<i16>),
    Int(Vec<i32>),
    Int64(Vec<i64>),
    Float(Vec<f32>),
    Double(Vec<f64>),
}

pub struct NCVariable {

}

#[derive(Debug)]
pub struct ParseError {
    reason: String,
}

impl ParseError {
    pub fn new(reason: &str) -> Self {
        ParseError {
            reason: String::from(reason),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.reason)
    }
}

impl Error for ParseError {
    fn description(&self) -> &str {
        self.reason.as_str()
    }
}

impl From<io::Error> for ParseError {
    fn from(e: io::Error) -> Self {
        ParseError {
            reason: e.to_string(),
        }
    }
}

impl From<FromUtf8Error> for ParseError {
    fn from(e: FromUtf8Error) -> Self {
        ParseError {
            reason: e.to_string(),
        }
    }
}

type Result<T> = result::Result<T, ParseError>;

pub struct NCFile {
    version: u8,
    numrecs: u32,
    dimensions: Vec<NCDimension>,
    attributes: Vec<NCAttribute>,
    variables: Vec<NCVariable>,
}

impl NCFile {
    pub fn new<R: io::Read>(r: &mut R) -> Result<Self> {
        let mut f = NCFile {
            version: 0,
            numrecs: 0,
            dimensions: Vec::new(),
            attributes: Vec::new(),
            variables: Vec::new(),
        };

        NCFile::validate_magic_number(r)?;
        f.version = read_u8(r)?;
        f.numrecs = read_u32(r)?;

        let dimflag = read_i32(r)? as u8;
        if dimflag == NC_DIMENSION {
            f.dimensions = NCFile::parse_dimlist(r)?;
        } else {
            // advance 4 bytes
            read_u32(r);
        }

        Ok(f)
    }

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = fs::File::open(path)?;
        NCFile::new(&mut file)
    }

    fn validate_magic_number<R: io::Read>(r: &mut R) -> Result<()> {
        let mut buf: [u8; 3] = [0; 3];
        
        r.read_exact(&mut buf).unwrap();
        let magic = String::from_utf8(buf.to_vec())?;

        if magic != MAGIC_NUMBER {
            Err(ParseError::new("incorrect magic number"))
        } else {
            Ok(())
        }
    }

    fn parse_dimlist<R: io::Read>(r: &mut R) -> Result<Vec<NCDimension>> {
        let len = read_u32(r)?;
        let mut dimlist: Vec<NCDimension> = Vec::new();

        for _ in 0..len {
            dimlist.push(NCFile::parse_dim(r)?);
        }

        Ok(dimlist)
    }

    fn parse_dim<R: io::Read>(r: &mut R) -> Result<NCDimension> {
        let len = read_u32(r)? as usize;

        // string length is rounded to the nearest 4 bytes
        let buflen = if len % 4 == 0 {
            len
        } else {
            len + (4 - (len % 4))
        };

        let mut namebuf = vec![0; buflen];

        r.read_exact(&mut namebuf)?;
        let name = String::from_utf8((namebuf[..len]).to_vec())?;
        let dimlen = read_u32(r)?;

        Ok(NCDimension {
            name,
            length: dimlen,
        })
    }
}


fn read_u8<R: io::Read>(r: &mut R) -> Result<u8> {
    let mut buf: [u8; 1] = [0; 1];
    r.read_exact(&mut buf)?;
    Ok(buf[0])
}

fn read_u32<R: io::Read>(r: &mut R) -> Result<u32> {
    let mut buf: [u8; 4] = [0; 4];
    r.read_exact(&mut buf)?;
    Ok(u32::from_be_bytes(buf))
}

fn read_i32<R: io::Read>(r: &mut R) -> Result<i32> {
    let mut buf: [u8; 4] = [0; 4];
    r.read_exact(&mut buf)?;
    Ok(i32::from_be_bytes(buf))
}

#[cfg(test)]
mod test {
    use std::fs;
    use super::*;

    const SAMPLE_FILE: &'static str = "./samples/sample1.nc";

    #[test]
    fn it_opens_from_file() {
        NCFile::open(SAMPLE_FILE).unwrap();
    }

    #[test]
    fn it_opens_from_reader() {
        let mut f = fs::File::open(SAMPLE_FILE).unwrap();
        NCFile::new(&mut f).unwrap();
    }

    #[test]
    fn it_parses_dimensions() {
        let f = NCFile::open(SAMPLE_FILE).unwrap();
        assert_eq!(f.dimensions[0].name, "longitude");
    }
}
