#![macro_escape]

use std::io::{File, BufferedReader};

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
            fn from_xml<'a>(iter:&'a mut ::xml::reader::Events<::std::io::BufferedReader<::std::io::File>>) -> Option<$Id> {
                let obj = $Id { ..Default::default() };

                fn inner<'a> (iter:&'a mut ::xml::reader::Events<::std::io::BufferedReader<::std::io::File>>, arg:&mut $Id, name:&str) {
                    match name {
                        $($(#[$Flag_field])* stringify!($Flag) => arg.$Flag  = ::fromxml::FromXml::from_xml(iter),)+
                        _ => ::fromxml::skip_node(iter),
                    };
                }

                return ::fromxml::collect(iter, obj, inner);
            }
        })+
    };
}

type XmlIter<'a> = Events<'a, BufferedReader<File>>;

pub fn collect<'a, T>(iter:&'a mut XmlIter, mut arg:T, back:for<'b> fn(&'b mut XmlIter, &mut T, &str)) -> Option<T> {
    loop {
        match iter.next() {
            Some(StartElement { name, attributes: _, namespace: _ }) => {
                back(iter, &mut arg, name.local_name.as_slice());
            }
            Some(EndElement { name: _ }) => break,
            Some(Error(e)) => {
                println!("Error: {}", e);
                return None;
            }
            Some(_) => {}
            None => return None
        }
    }

    return Some(arg);
}

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

pub trait FromXml {
    fn from_xml<'a>(iter:&'a mut XmlIter) -> Option<Self>;
}

impl<T:FromXml> FromXml for Vec<T> {
    fn from_xml<'a>(iter:&'a mut XmlIter) -> Option<Vec<T>> {
        let mut ret:Vec<T> = vec![];
        loop {
            match iter.next() {
                Some(StartElement { name: _, attributes: _, namespace: _ }) => {
                    ret.push(FromXml::from_xml(iter).unwrap());
                }
                Some(EndElement { name: _ }) => break,
                Some(..) => (),
                None => return None
            }
        }
        Some(ret)
    }
}

impl FromXml for uint {
    fn from_xml<'a>(iter:&'a mut XmlIter) -> Option<uint> {
        FromXml::from_xml(iter).and_then(|s: String| from_str(&*s))
    }
}

impl FromXml for String {
    fn from_xml<'a>(iter:&'a mut XmlIter) -> Option<String> {
        let mut str = "".to_string();
        loop {
            match iter.next() {
                Some(Characters(text)) => str.push_str(text.as_slice()),
                Some(_) => break,
                None => return None,
            }
        }
        Some(str)
    }
}

pub fn parse_root<'a, T:FromXml>(iter:&'a mut XmlIter) -> Option<T> {
    loop {
        match iter.next() {
            Some(StartElement { name: _, attributes: _, namespace: _ }) => {
                return FromXml::from_xml(iter);
            }
            Some(Error(e)) => {
                println!("Error: {}", e);
                return None;
            }
            Some(..) => {}
            None => return None
        }
    }
}
