#![feature(phase, plugin_registrar, macro_rules, quote)]

extern crate syntax;
extern crate rustc;

use std::iter::Peekable;
use std::slice::Items;
use std::rc::Rc;
use std::num::{FromStrRadix};

use syntax::ast;
use syntax::ast::Delimited;
use syntax::ast::TokenTree::{TtToken, TtDelimited};
use syntax::codemap;
use syntax::parse::token::Token;
use syntax::parse::token::Lit;
use syntax::parse::token::{str_to_ident};
use syntax::ext::base::{ExtCtxt, ItemModifier, DummyResult, MacResult, MacExpr, MacItems};
// NB. this is important or the method calls don't work
use syntax::ext::build::AstBuilder;
use syntax::parse::token;
use syntax::util::small_vector::SmallVector;

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut rustc::plugin::Registry) {
    reg.register_macro("foo", expand);
}

type AstIterator<'a> = Peekable<&'a ast::TokenTree, Items<'a, ast::TokenTree>>;

fn read_ident<'a>(iter:&mut AstIterator<'a>) -> Result<String, String> {
	match iter.peek() {
		Some(&&TtToken(_, Token::Ident(value, kind))) => {
			iter.next();
			Ok(value.as_str().to_string())
		},
		_ => {
			Err(format!("expecting identifier"))
		}
	}
}

fn read_brackets<'a>(iter:&mut AstIterator<'a>) -> Result<Rc<Delimited>, String> {
	match iter.peek() {
		Some(&&TtDelimited(_, ref treeptr)) => {
			iter.next();
			Ok(treeptr.clone())
		},
		_ => {
			Err(format!("expecting identifier"))
		}
	}
}

fn read_tree<'a>(item:&'a Delimited) -> AstIterator<'a> {
	item.tts.iter().peekable()
}

fn read_int<'a>(iter:&mut AstIterator<'a>) -> Result<int, String> {
	match iter.peek() {
		Some(&&TtToken(_, Token::Literal(Lit::Integer(value), kind))) => {
			iter.next();
			match if value.as_str().contains_char('x') {
                FromStrRadix::from_str_radix(value.as_str().slice_from(2), 16)
            } else {
                from_str(value.as_str())
            } {
				Some(value) => Ok(value),
				None => Err(format!("invalid integer")),
			}
		},
		_ => {
			Err(format!("expecting integer"))
		}
	}
}

fn read_token<'a>(iter:&mut AstIterator<'a>, t:Token) -> Result<(), String> {
	match (iter.peek(), &t) {
		(Some(&&TtToken(_, Token::FatArrow)), &Token::FatArrow) => {
			iter.next();
			Ok(())
		},
		_ => {
			Err(format!("expecting simple token {}", t))
		}
	}
}

#[deriving(Show)]
struct Field {
	name:String,
	width:uint,
}

fn parse_field<'a>(iter:&mut AstIterator<'a>) -> Result<Field, String> {
	try!(read_int(iter));
	try!(read_token(iter, Token::FatArrow));

	let access = try!(read_ident(iter));
	let delimiter = try!(read_brackets(iter));
	let sub = &mut read_tree(&*delimiter);
	let subscript = try!(read_int(sub));
	let name = try!(read_ident(iter));

	println!("{} {} {}", access, subscript, name);

	let a = Field {
		name: name,
		width: subscript as uint,
	};

	return Ok(a);
}

fn expand<'a>(cx: &'a mut ExtCtxt, sp: codemap::Span, tokens: &[ast::TokenTree]) -> Box<MacResult + 'a> {
	let mut result = vec![];

	// cx.span_err(sp, "dummy is only permissible on functions");

	let mut iter = tokens.iter().peekable();

	    for i in tokens.iter() {
	    	// println!("{}", i);
	    }

	match parse_field(&mut iter) {
		Ok(field) => {
			println!("{}", field);
			let n = str_to_ident(field.name.as_slice());
			let width = field.width;
			result.push(quote_item!(cx, const $n:RegField = RegField { width: $width };).unwrap());
		},
		Err(err) => {
			cx.span_err(sp, err.as_slice());
		}
	}

	// if false {

	return MacItems::new(SmallVector::many(result).into_iter());


	//     // result.push(quote_item!(cx, struct CustomStruct;).unwrap());
	// // }

	// return MacItems::new(SmallVector::many(result).into_iter());
    // MacExpr::new(quote_expr!(&mut *cx,
    // 	println!("fix me");
    // ))
}
