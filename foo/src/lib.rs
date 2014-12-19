#![feature(phase, plugin_registrar, macro_rules, quote)]

#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unused_imports)]

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
use syntax::parse::token;
use syntax::util::small_vector::SmallVector;

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut rustc::plugin::Registry) {
    reg.register_macro("foo", expand);
}

type AstIterator<'a> = Peekable<&'a ast::TokenTree, Items<'a, ast::TokenTree>>;

fn is_ident(t:Option<&&ast::TokenTree>) -> bool {
	match t {
		Some(&&TtToken(_, Token::Ident(..))) => true,
		_ => false,
	}
}

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

fn is_int(t:Option<&&ast::TokenTree>) -> bool {
	match t {
		Some(&&TtToken(_, Token::Literal(Lit::Integer(..), _))) => true,
		_ => false,
	}
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

fn is_token(t:&Token, u:&Token) -> bool {
	match (t, u) {
		(&Token::FatArrow, &Token::FatArrow) |
		(&Token::Comma, &Token::Comma) => {
			true
		}
		_ => false
	}
}

fn read_token<'a>(iter:&mut AstIterator<'a>, t:Token) -> Result<(), String> {
	match iter.peek() {
		Some(&&TtToken(_, ref u)) => {
			if is_token(&t, u) {
				iter.next();
				return Ok(())
			}
		}
		_ => {}
	}
	Err(format!("expecting simple token {}", t))
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

fn parse_fields<'a>(iter:&mut AstIterator<'a>) -> Result<Vec<Field>, String> {
	let mut out:Vec<Field> = vec![];
	while is_int(iter.peek()) {
		out.push(try!(parse_field(iter)));
		match read_token(iter, Token::Comma) {
			Ok(..) => (),
			Err(..) => { break }
		}
	}
	Ok(out)
}

fn expand<'a>(cx: &'a mut ExtCtxt, sp: codemap::Span, tokens: &[ast::TokenTree]) -> Box<MacResult + 'a> {
	let mut result = vec![];

	// cx.span_err(sp, "dummy is only permissible on functions");

	let mut iter = tokens.iter().peekable();

    // for i in tokens.iter() {
    // 	println!("{}", i);
    // }

    for field in parse_fields(&mut iter).unwrap().iter() {
		println!("{}", field);
		let n = str_to_ident(field.name.as_slice());
		let width = field.width;
		let tokens = quote_tokens!(cx, const $n:RegField = RegField { width: $width };);
		result.push(quote_item!(cx, $tokens).unwrap());
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
