#![macro_escape]

use std::io::{File, BufferedReader};
use std::str::FromStr;

use xml::reader::Events;
use xml::reader::events::XmlEvent::*;

#[macro_export]
macro_rules! deriving_fromxml {
    (
        $($(#[$attr:meta])* struct $Id:ident {
            $($(#[$Flag_field:meta])* $Flag:ident:$T:ty),+,
        })+
    ) => {
        $($(#[$attr])*
        #[deriving(Default, Show)]
        #[allow(non_snake_case)]
        struct $Id {
            $($(#[$Flag_field])* $Flag:$T,)+
        }

        impl ::fromxml::FromXml for $Id {
            fn from_xml<'a>(iter:&'a mut ::xml::reader::Events<::std::io::BufferedReader<::std::io::File>>) -> Result<$Id, ()> {
                use xml::reader::events::XmlEvent::*;
                let mut obj = $Id { ..Default::default() };

                loop {
                    match iter.next() {
                        Some(StartElement { name, attributes: _, namespace: _ }) => {
                            match name.local_name.as_slice() {
                                $(stringify!($Flag) => obj.$Flag = try!(::fromxml::FromXml::from_xml(iter)),)+
                                _ => ::fromxml::skip_node(iter),
                            }
                        }
                        Some(EndElement { name: _ }) => break,
                        Some(Error(e)) => {
                            println!("Error: {}", e);
                            return Err(())
                        }
                        Some(_) => {}
                        None => return Err(())
                    }
                }

                Ok(obj)
            }
        })+
    };
}

type XmlIter<'a> = Events<'a, BufferedReader<File>>;

pub fn skip_node<'a>(iter:&'a mut XmlIter) {
    let mut depth:uint = 1;
    loop {
        match iter.next() {
            Some(StartElement { name: _, attributes: _, namespace: _ }) => {
                depth = depth + 1;
            }
            Some(EndElement { name: _ }) => {
                depth = depth - 1;
                if depth == 0 {
                    return;
                }
            },
            Some(Error(e)) => {
                println!("Error: {}", e);
                return;
            }
            Some(..) => {}
            None => return

        }
    }
}

pub type XmlError = ();

pub trait FromXml {
    fn from_xml<'a>(iter:&'a mut XmlIter) -> Result<Self, XmlError>;
}

impl<T:FromXml> FromXml for Vec<T> {
    fn from_xml<'a>(iter:&'a mut XmlIter) -> Result<Vec<T>, XmlError> {
        let mut ret:Vec<T> = vec![];
        loop {
            match iter.next() {
                Some(StartElement { name: _, attributes: _, namespace: _ }) => {
                    ret.push(try!(FromXml::from_xml(iter)));
                }
                Some(EndElement { name: _ }) => break,
                Some(..) => (),
                None => return Err(())
            }
        }
        Ok(ret)
    }
}

impl<T:FromXml> FromXml for Option<T> {
    fn from_xml<'a>(iter: &'a mut XmlIter) -> Result<Option<T>, XmlError> {
        FromXml::from_xml(iter).map(Some)
    }
}

impl<T> FromXml for T where T: FromStr {
    fn from_xml<'a>(iter:&'a mut XmlIter) -> Result<T, XmlError> {
        let mut s = "".to_string();
        loop {
            match iter.next() {
                Some(Characters(text)) => s.push_str(text.as_slice()),
                Some(_) => break,
                None => return Err(()),
            }
        }
        from_str(&*s).ok_or(())
    }
}

pub fn parse_root<'a, T:FromXml>(iter:&'a mut XmlIter) -> Result<T, XmlError> {
    loop {
        match iter.next() {
            Some(StartElement { name: _, attributes: _, namespace: _ }) => {
                return FromXml::from_xml(iter);
            }
            Some(Error(e)) => {
                println!("Error: {}", e);
                return Err(());
            }
            Some(..) => {}
            None => return Err(())
        }
    }
}
