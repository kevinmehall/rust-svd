#![macro_escape]

use std::io::{File, BufferedReader};
use std::str::FromStr;

use xml::reader::Events;
use xml::reader::events::XmlEvent;
use xml::reader::events::XmlEvent::*;
use xml::common::Attribute;

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
            fn from_xml(iter:&mut ::fromxml::XmlIter) -> Result<$Id, ()> {
                let mut obj = $Id { ..Default::default() };

                try!(iter.each_child(|iter|{
                    match iter.tag_name() {
                        $(stringify!($Flag) => obj.$Flag = try!(::fromxml::FromXml::from_xml(iter)),)+
                        _ => try!(iter.skip_node()),
                    };
                    Ok(())
                }));

                Ok(obj)
            }
        })+
    };
}

pub struct XmlIter<'a> {
    iter: Events<'a, BufferedReader<File>>,
    stack: Vec<(String, Vec<Attribute>)>,
}

impl<'a> Iterator<XmlEvent> for XmlIter<'a> {
    fn next(&mut self) -> Option<XmlEvent> {
        let event = self.iter.next();

        match event {
            Some(StartElement { ref name, ref attributes , namespace: _ }) => {
                self.stack.push((name.local_name.clone(), attributes.clone()));
            }
            Some(EndElement { ref name }) => {
                assert_eq!(name.local_name.as_slice(), self.tag_name());
                self.stack.pop();
            }
            _ => {}
        }

        event
    }
}

impl<'a> XmlIter<'a> {
    pub fn new(mut iter: Events<'a, BufferedReader<File>>) -> Result<XmlIter<'a>, XmlError> {
        loop {
            match iter.next() {
                Some(StartElement { name, attributes, namespace: _ }) => {
                    return Ok(XmlIter {
                        iter: iter,
                        stack: vec![(name.local_name, attributes)]
                    });
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

    pub fn tag_name(&self) -> &str {
        self.stack.last().unwrap().0.as_slice()
    }

    pub fn attributes(&self) -> &[Attribute] {
        self.stack.last().unwrap().1.as_slice()
    }

    pub fn skip_node(&mut self) -> Result<(), XmlError> {
        let depth = self.stack.len();
        while self.stack.len() >= depth {
            self.next();
        }
        Ok(())
    }

    pub fn each_child<F: FnMut(&mut XmlIter) -> Result<(), XmlError>>(&mut self, mut f: F) -> Result<(), XmlError> {
        let depth = self.stack.len();
        while self.stack.len() >= depth {
            match self.next() {
                Some(StartElement{..}) => {
                    try!(f(self))
                }
                Some(Error(e)) => {
                    println!("Error: {}", e);
                    return Err(());
                }
                Some(_) =>  {}
                None => return Err(()),
            }
        }
        Ok(())
    }

    pub fn inner_text(&mut self) -> Result<String, XmlError> {
        let depth = self.stack.len();
        let mut s = "".into_string();
        while self.stack.len() >= depth {
            match self.next() {
                Some(Characters(text)) => s.push_str(text.as_slice()),
                Some(Error(e)) => {
                    println!("Error: {}", e);
                    return Err(());
                }
                Some(_) =>  {}
                None => return Err(()),
            }
        }
        Ok(s)
    }
}


pub type XmlError = ();

pub trait FromXml {
    fn from_xml(iter:&mut XmlIter) -> Result<Self, XmlError>;
}

impl<T:FromXml> FromXml for Vec<T> {
    fn from_xml(iter:&mut XmlIter) -> Result<Vec<T>, XmlError> {
        let mut ret:Vec<T> = vec![];
        try!(iter.each_child(|iter| {
            ret.push(try!(FromXml::from_xml(iter)));
            Ok(())
        }));
        Ok(ret)
    }
}

impl<T:FromXml> FromXml for Option<T> {
    fn from_xml(iter: &mut XmlIter) -> Result<Option<T>, XmlError> {
        FromXml::from_xml(iter).map(Some)
    }
}

impl<T> FromXml for T where T: FromStr {
    fn from_xml(iter: &mut XmlIter) -> Result<T, XmlError> {
        let s = try!(iter.inner_text());
        from_str(s.as_slice()).ok_or(())
    }
}
