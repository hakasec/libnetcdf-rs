use std::io;
use std::fs;
use std::fmt;
use std::result;
use std::error::Error;
use std::path::Path;
use std::convert::From;
use std::string::FromUtf8Error;

use crate::consts::*;

#[derive(Debug)]
pub struct NCDimension {
    name: String,
    length: u32,
}

#[derive(Debug)]
pub enum NCAttribute {
    Byte(NCAttributeContainer<u8>),
    Char(NCAttributeContainer<char>),
    Short(NCAttributeContainer<i16>),
    Int(NCAttributeContainer<i32>),
    Float(NCAttributeContainer<f32>),
    Double(NCAttributeContainer<f64>),
}

pub struct NCAttributeContainer<T> {
    name: String,
    values: Vec<T>,
}

impl<T> NCAttributeContainer<T> {
    pub fn new(name: &str, values: Vec<T>) -> Self {
        Self {
            name: name.to_string(),
            values: values,
        }
    }
}

impl fmt::Display for NCAttributeContainer<char> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s: String = self.values.iter().collect();
        write!(f, "{}", s)
    }
}

impl fmt::Debug for NCAttributeContainer<char> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s: String = self.values.iter().collect();
        f.debug_struct(stringify!(self))
            .field("name", &self.name)
            .field("values", &s)
            .finish()
    }
}

impl fmt::Debug for NCAttributeContainer<u8> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(stringify!(self))
            .field("name", &self.name)
            .field("values", &self.values)
            .finish()
    }
}

impl fmt::Debug for NCAttributeContainer<i16> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(stringify!(self))
            .field("name", &self.name)
            .field("values", &self.values)
            .finish()
    }
}

impl fmt::Debug for NCAttributeContainer<i32> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(stringify!(self))
            .field("name", &self.name)
            .field("values", &self.values)
            .finish()
    }
}

impl fmt::Debug for NCAttributeContainer<f32> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(stringify!(self))
            .field("name", &self.name)
            .field("values", &self.values)
            .finish()
    }
}

impl fmt::Debug for NCAttributeContainer<f64> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(stringify!(self))
            .field("name", &self.name)
            .field("values", &self.values)
            .finish()
    }
}

#[derive(Debug)]
pub struct NCVariable {

}

#[derive(Debug)]
pub struct ParseError {
    reason: String,
}

impl ParseError {
    pub fn new(reason: &str) -> Self {
        Self {
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
        Self {
            reason: e.to_string(),
        }
    }
}

impl From<FromUtf8Error> for ParseError {
    fn from(e: FromUtf8Error) -> Self {
        Self {
            reason: e.to_string(),
        }
    }
}

type Result<T> = result::Result<T, ParseError>;

#[derive(Debug)]
pub struct NCFile {
    version: u8,
    numrecs: u32,
    dimensions: Vec<NCDimension>,
    attributes: Vec<NCAttribute>,
    variables: Vec<NCVariable>,
}

impl NCFile {

    pub fn new<R: io::Read>(r: &mut R) -> Result<Self> {
        let mut f = Self {
            version: 0,
            numrecs: 0,
            dimensions: Vec::new(),
            attributes: Vec::new(),
            variables: Vec::new(),
        };

        Self::validate_magic_number(r)?;
        f.version = read_u8(r)?;
        f.numrecs = read_u32(r)?;

        let dimflag = read_u32(r)? as u8;
        if dimflag == NC_DIMENSION {
            f.dimensions = Self::parse_dimlist(r)?;
        } else {
            // advance 4 bytes
            read_u32(r)?;
        }

        let attrflag = read_u32(r)? as u8;
        if attrflag == NC_ATTRIBUTE {
            f.attributes = NCFile::parse_attrlist(r)?;
        } else {
            // advance 4 bytes
            read_u32(r)?;
        }

        Ok(f)
    }

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = fs::File::open(path)?;
        Self::new(&mut file)
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
        let name = read_string(r)?;
        let dimlen = read_u32(r)?;

        Ok(NCDimension {
            name,
            length: dimlen,
        })
    }

    fn parse_attrlist<R: io::Read>(r: &mut R) -> Result<Vec<NCAttribute>> {
        let len = read_u32(r)?;
        let mut attrlist: Vec<NCAttribute> = Vec::new();

        for _ in 0..len {
            attrlist.push(NCFile::parse_attr(r)?);
        }

        Ok(attrlist)
    }

    fn parse_attr<R: io::Read>(r: &mut R) -> Result<NCAttribute> {
        let name = &read_string(r)?;
        let nctype = read_u32(r)? as u8;

        Ok(match nctype {
            NC_BYTE => {
                NCAttribute::Byte(
                    NCAttributeContainer::new(name, read_bytes(r)?)
                )
            },
            NC_CHAR => {
                let s = read_string(r)?;

                NCAttribute::Char(
                    NCAttributeContainer::new(name, s.chars().collect())
                )
            },
            NC_SHORT => {
                NCAttribute::Short(
                    NCAttributeContainer::new(name, read_i16_list(r)?)
                )
            },
            NC_INT => {
                NCAttribute::Int(
                    NCAttributeContainer::new(name, read_i32_list(r)?)
                )
            },
            NC_FLOAT => {
                NCAttribute::Float(
                    NCAttributeContainer::new(name, read_f32_list(r)?)
                )
            },
            NC_DOUBLE => {
                NCAttribute::Double(
                    NCAttributeContainer::new(name, read_f64_list(r)?)
                )
            }

            _ => return Err(ParseError::new("unknown type")),
        })
    }
}


fn read_u8<R: io::Read>(r: &mut R) -> Result<u8> {
    let mut buf: [u8; 1] = [0; 1];
    r.read_exact(&mut buf)?;
    Ok(buf[0])
}

fn read_i16<R: io::Read>(r: &mut R) -> Result<i16> {
    let raw = read_bytes_padded(r, 2)?;
    let buf: [u8; 2] = [raw[0], raw[1]];
    Ok(i16::from_be_bytes(buf))
}

fn read_i16_list<R: io::Read>(r: &mut R) -> Result<Vec<i16>> {
    let len = read_u32(r)? as usize;
    let mut vals = Vec::new();

    for _ in 0..len {
        vals.push(read_i16(r)?);
    }

    Ok(vals)
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

fn read_i32_list<R: io::Read>(r: &mut R) -> Result<Vec<i32>> {
    let len = read_u32(r)? as usize;
    let mut vals = Vec::new();

    for _ in 0..len {
        vals.push(read_i32(r)?);
    }

    Ok(vals)
}

fn read_f32<R: io::Read>(r: &mut R) -> Result<f32> {
    let mut buf: [u8; 4] = [0; 4];
    r.read_exact(&mut buf)?;
    Ok(f32::from_be_bytes(buf))
}

fn read_f32_list<R: io::Read>(r: &mut R) -> Result<Vec<f32>> {
    let len = read_u32(r)? as usize;
    let mut vals = Vec::new();

    for _ in 0..len {
        vals.push(read_f32(r)?);
    }

    Ok(vals)
}

fn read_f64<R: io::Read>(r: &mut R) -> Result<f64> {
    let mut buf: [u8; 8] = [0; 8];
    r.read_exact(&mut buf)?;
    Ok(f64::from_be_bytes(buf))
}

fn read_f64_list<R: io::Read>(r: &mut R) -> Result<Vec<f64>> {
    let len = read_u32(r)? as usize;
    let mut vals = Vec::new();

    for _ in 0..len {
        vals.push(read_f64(r)?);
    }

    Ok(vals)
}

fn read_bytes_padded<R: io::Read>(r: &mut R, len: usize) -> Result<Vec<u8>> {
    // string length is rounded to the nearest 4 bytes
    let buflen = if len % 4 == 0 {
        len
    } else {
        len + (4 - (len % 4))
    };

    let mut buf = vec![0; buflen];
    r.read_exact(&mut buf)?;
    Ok(buf)
}

fn read_bytes<R: io::Read>(r: &mut R) -> Result<Vec<u8>> {
    let len = read_u32(r)? as usize;
    let buf = read_bytes_padded(r, len)?;
    Ok(buf[..len].to_vec())
}

fn read_string<R: io::Read>(r: &mut R) -> Result<String> {
    let strbuf = read_bytes(r)?;
    Ok(String::from_utf8(strbuf)?)
}

#[cfg(test)]
mod test {
    use std::fs;
    use super::*;

    const SAMPLE_FILE: &'static str = "./samples/sample1.nc";

    fn open_sample() -> NCFile {
        NCFile::open(SAMPLE_FILE).unwrap()
    }

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
        let f = open_sample();
        assert_eq!(f.dimensions[0].name, "longitude");
    }

    #[test]
    fn it_parses_attributes() {
        let f = open_sample();
        println!("{:?}", f.attributes);
        if let NCAttribute::Char(c) = &f.attributes[0] {
            assert_eq!(c.name, "Conventions");
            assert_eq!(c.to_string(), "CF-1.6");
        } else {
            panic!("first attribute isn't Char");
        }
    }
}
