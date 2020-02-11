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
        f.debug_struct("NCAttributeContainer<char>")
            .field("name", &self.name)
            .field("values", &s)
            .finish()
    }
}

impl fmt::Debug for NCAttributeContainer<u8> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NCAttributeContainer<u8>")
            .field("name", &self.name)
            .field("values", &self.values)
            .finish()
    }
}

impl fmt::Debug for NCAttributeContainer<i16> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NCAttributeContainer<i16>")
            .field("name", &self.name)
            .field("values", &self.values)
            .finish()
    }
}

impl fmt::Debug for NCAttributeContainer<i32> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NCAttributeContainer<i32>")
            .field("name", &self.name)
            .field("values", &self.values)
            .finish()
    }
}

impl fmt::Debug for NCAttributeContainer<f32> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NCAttributeContainer<f32>")
            .field("name", &self.name)
            .field("values", &self.values)
            .finish()
    }
}

impl fmt::Debug for NCAttributeContainer<f64> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NCAttributeContainer<f64>")
            .field("name", &self.name)
            .field("values", &self.values)
            .finish()
    }
}

#[derive(Debug)]
pub enum NCVariable {
    Byte(NCVariableContainer<u8>),
    Char(NCVariableContainer<char>),
    Short(NCVariableContainer<i16>),
    Int(NCVariableContainer<i32>),
    Float(NCVariableContainer<f32>),
    Double(NCVariableContainer<f64>),
}

#[derive(Debug)]
pub struct NCVariableContainer<T> {
    name: String,
    dimids: Vec<u32>,
    attributes: Vec<NCAttribute>,
    data: Vec<T>,
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
    pub fn new<R: io::Read + io::Seek>(r: &mut R) -> Result<Self> {
        let mut f = Self {
            version: 0,
            numrecs: 0,
            dimensions: Vec::new(),
            attributes: Vec::new(),
            variables: Vec::new(),
        };

        f.validate_magic_number(r)?;
        f.version = read_u8(r)?;
        f.numrecs = read_u32(r)?;

        let dimflag = read_u32(r)? as u8;
        if dimflag == NC_DIMENSION {
            f.dimensions = f.parse_dimlist(r)?;
        } else {
            // advance 4 bytes
            r.seek(io::SeekFrom::Current(4))?;
        }

        let attrflag = read_u32(r)? as u8;
        if attrflag == NC_ATTRIBUTE {
            f.attributes = f.parse_attrlist(r)?;
        } else {
            // advance 4 bytes
            r.seek(io::SeekFrom::Current(4))?;
        }

        let varflag = read_u32(r)? as u8;
        if varflag == NC_VARIABLE {
            f.variables = f.parse_varlist(r)?;
        } else {
            // advance 4 bytes
            r.seek(io::SeekFrom::Current(4))?;
        }

        Ok(f)
    }

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = fs::File::open(path)?;
        Self::new(&mut file)
    }

    fn validate_magic_number<R: io::Read>(&self, r: &mut R) -> Result<()> {
        let mut buf: [u8; 3] = [0; 3];
        
        r.read_exact(&mut buf).unwrap();
        let magic = String::from_utf8(buf.to_vec())?;

        if magic != MAGIC_NUMBER {
            Err(ParseError::new("incorrect magic number"))
        } else {
            Ok(())
        }
    }

    fn parse_dimlist<R: io::Read>(&self, r: &mut R) -> Result<Vec<NCDimension>> {
        let len = read_u32(r)?;
        let mut dimlist: Vec<NCDimension> = Vec::new();

        for _ in 0..len {
            dimlist.push(self.parse_dim(r)?);
        }

        Ok(dimlist)
    }

    fn parse_dim<R: io::Read>(&self, r: &mut R) -> Result<NCDimension> {
        let name = read_string(r)?;
        let dimlen = read_u32(r)?;

        Ok(NCDimension {
            name,
            length: dimlen,
        })
    }

    fn parse_attrlist<R: io::Read>(&self, r: &mut R) -> Result<Vec<NCAttribute>> {
        let len = read_u32(r)?;
        let mut attrlist: Vec<NCAttribute> = Vec::new();

        for _ in 0..len {
            attrlist.push(self.parse_attr(r)?);
        }

        Ok(attrlist)
    }

    fn parse_attr<R: io::Read>(&self, r: &mut R) -> Result<NCAttribute> {
        let name = &read_string(r)?;
        let nctype = read_u32(r)? as u8;

        Ok(match nctype {
            NC_BYTE => {
                let len = read_u32(r)? as usize;

                NCAttribute::Byte(
                    NCAttributeContainer::new(name, read_bytes(r, len)?)
                )
            },
            NC_CHAR => {
                let s = read_string(r)?;

                NCAttribute::Char(
                    NCAttributeContainer::new(name, s.chars().collect())
                )
            },
            NC_SHORT => {
                let len = read_u32(r)? as usize;

                NCAttribute::Short(
                    NCAttributeContainer::new(name, read_i16_padded_list(r, len)?)
                )
            },
            NC_INT => {
                let len = read_u32(r)? as usize;

                NCAttribute::Int(
                    NCAttributeContainer::new(name, read_i32_list(r, len)?)
                )
            },
            NC_FLOAT => {
                let len = read_u32(r)? as usize;

                NCAttribute::Float(
                    NCAttributeContainer::new(name, read_f32_list(r, len)?)
                )
            },
            NC_DOUBLE => {
                let len = read_u32(r)? as usize;

                NCAttribute::Double(
                    NCAttributeContainer::new(name, read_f64_list(r, len)?)
                )
            }

            _ => return Err(ParseError::new("unknown type")),
        })
    }

    fn parse_varlist<R: io::Read + io::Seek>(&self, r: &mut R) -> Result<Vec<NCVariable>> {
        let len = read_u32(r)?;
        let mut varlist: Vec<NCVariable> = Vec::new();

        for _ in 0..len {
            varlist.push(self.parse_var(r)?);
        }

        Ok(varlist)
    }

    fn parse_var<R: io::Read + io::Seek>(&self, r: &mut R) -> Result<NCVariable> {
        let name = read_string(r)?;
        let dimlen = read_u32(r)?;
        let mut dimids = Vec::new();
        
        for _ in 0..dimlen {
            dimids.push(read_u32(r)?);
        }

        // next byte is attr flag
        r.seek(io::SeekFrom::Current(4))?;
        let attributes = self.parse_attrlist(r)?;

        let nctype = read_u32(r)? as u8;
        let vsize = read_u32(r)? as usize;
        let offset = if self.version == 0x1 {
            read_u32(r)? as u64
        } else {
            read_u64(r)?
        };

        // keep track of the old stream position
        let was = r.seek(io::SeekFrom::Current(0))?;
        // seek to offset
        r.seek(io::SeekFrom::Start(offset))?;

        let var = match nctype {
            NC_BYTE => {
                NCVariable::Byte(
                    NCVariableContainer::<u8> {
                        name,
                        dimids,
                        attributes,
                        data: read_bytes(r, vsize)?,
                    }
                )
            },
            NC_CHAR => {
                let raw = read_bytes(r, vsize)?;

                NCVariable::Char(
                    NCVariableContainer::<char> {
                        name,
                        dimids,
                        attributes,
                        data: String::from_utf8(raw)?.chars().collect(),
                    }
                )
            },
            NC_SHORT => {
                NCVariable::Short(
                    NCVariableContainer::<i16> {
                        name,
                        dimids,
                        attributes,
                        data: read_i16_list(r, vsize / 2)?,
                    }
                )
            },
            NC_INT => {
                NCVariable::Int(
                    NCVariableContainer::<i32> {
                        name,
                        dimids,
                        attributes,
                        data: read_i32_list(r, vsize / 4)?,
                    }
                )
            },
            NC_FLOAT => {
                NCVariable::Float(
                    NCVariableContainer::<f32> {
                        name,
                        dimids,
                        attributes,
                        data: read_f32_list(r, vsize / 4)?,
                    }
                )
            },
            NC_DOUBLE => {
                NCVariable::Double(
                    NCVariableContainer::<f64> {
                        name,
                        dimids,
                        attributes,
                        data: read_f64_list(r, vsize / 8)?,
                    }
                )
            },

            _ => return Err(ParseError::new("unknown type")),
        };

        // seek back to end of variable def
        r.seek(io::SeekFrom::Start(was))?;

        Ok(var)
    }
}

fn read_u8<R: io::Read>(r: &mut R) -> Result<u8> {
    let mut buf: [u8; 1] = [0; 1];
    r.read_exact(&mut buf)?;
    Ok(buf[0])
}

fn read_i16<R: io::Read>(r: &mut R) -> Result<i16> {
    let mut buf: [u8; 2] = [0; 2];
    r.read_exact(&mut buf)?;
    Ok(i16::from_be_bytes(buf))
}

fn read_i16_list<R: io::Read>(r: &mut R, len: usize) -> Result<Vec<i16>> {
    let mut vals = Vec::new();

    for _ in 0..len {
        let v = read_i16(r)?;
        vals.push(v);
    }

    Ok(vals)
}

fn read_i16_padded<R: io::Read>(r: &mut R) -> Result<i16> {
    let raw = read_bytes_padded(r, 2)?;
    let buf: [u8; 2] = [raw[0], raw[1]];
    Ok(i16::from_be_bytes(buf))
}

fn read_i16_padded_list<R: io::Read>(r: &mut R, len: usize) -> Result<Vec<i16>> {
    let mut vals = Vec::new();

    for _ in 0..len {
        let v = read_i16_padded(r)?;
        vals.push(v);
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

fn read_i32_list<R: io::Read>(r: &mut R, len: usize) -> Result<Vec<i32>> {
    let mut vals = Vec::new();

    for _ in 0..len {
        vals.push(read_i32(r)?);
    }

    Ok(vals)
}

fn read_u64<R: io::Read>(r: &mut R) -> Result<u64> {
    let mut buf: [u8; 8] = [0; 8];
    r.read_exact(&mut buf)?;
    Ok(u64::from_be_bytes(buf))
}

fn read_f32<R: io::Read>(r: &mut R) -> Result<f32> {
    let mut buf: [u8; 4] = [0; 4];
    r.read_exact(&mut buf)?;
    Ok(f32::from_be_bytes(buf))
}

fn read_f32_list<R: io::Read>(r: &mut R, len: usize) -> Result<Vec<f32>> {
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

fn read_f64_list<R: io::Read>(r: &mut R, len: usize) -> Result<Vec<f64>> {
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

fn read_bytes<R: io::Read>(r: &mut R, len: usize) -> Result<Vec<u8>> {
    let buf = read_bytes_padded(r, len)?;
    Ok(buf[..len].to_vec())
}

fn read_string<R: io::Read>(r: &mut R) -> Result<String> {
    let len = read_u32(r)? as usize;
    let strbuf = read_bytes(r, len)?;
    Ok(String::from_utf8(strbuf)?)
}

#[cfg(test)]
mod test {
    use std::fs;
    use super::*;

    const SAMPLE_FILE_1: &'static str = "./samples/sample1.nc";
    const SAMPLE_FILE_2: &'static str = "./samples/sample2.nc";

    fn open_sample1() -> NCFile {
        NCFile::open(SAMPLE_FILE_1).unwrap()
    }

    fn open_sample2() -> NCFile {
        NCFile::open(SAMPLE_FILE_2).unwrap()
    }

    #[test]
    fn it_opens_from_file() {
        NCFile::open(SAMPLE_FILE_1).unwrap();
    }

    #[test]
    fn it_opens_from_reader() {
        let mut f = fs::File::open(SAMPLE_FILE_1).unwrap();
        NCFile::new(&mut f).unwrap();
    }

    #[test]
    fn it_parses_dimensions() {
        let f1 = open_sample1();
        let f2 = open_sample2();

        assert_eq!(f1.dimensions[0].name, "longitude");
        assert_eq!(f2.dimensions[1].name, "latitude");
    }

    #[test]
    fn it_parses_attributes() {
        let f = open_sample1();

        if let NCAttribute::Char(c) = &f.attributes[0] {
            assert_eq!(c.name, "Conventions");
            assert_eq!(c.to_string(), "CF-1.6");
        } else {
            panic!("first attribute isn't Char");
        }
    }

    #[test]
    fn it_parses_variables() {
        let f = open_sample1();

        if let NCVariable::Float(n) = &f.variables[0] {
            assert_eq!(n.name, "longitude");
            assert_eq!(n.dimids[0], 0);
            if let NCAttribute::Char(c) = &n.attributes[0] {
                assert_eq!(c.name, "units");
            } else {
                panic!("first attribute of first variable isn't Char");
            }
        } else {
            panic!("first variable isn't Float");
        }
    }
}
