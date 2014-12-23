//! Generic AST reader.

#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unused_imports)]

extern crate syntax;
extern crate rustc;

use std::iter::Peekable;
use std::slice::Items;
use std::rc::Rc;
use std::num::{FromStrRadix};
use std::iter::Map;

use syntax::ast;
use syntax::ast::Delimited;
use syntax::ast::TokenTree::{TtToken, TtDelimited};
use syntax::codemap;
use syntax::parse::token::Token;
use syntax::parse::token::Lit;
use syntax::parse::token::{str_to_ident};
use syntax::ext::base::{ExtCtxt, ItemModifier, DummyResult, MacResult, MacExpr, MacItems};
use syntax::parse::token;
use syntax::util::small_vector::SmallVector;

fn is_ident(t:Option<&&ast::TokenTree>) -> bool {
	match t {
		Some(&&TtToken(_, Token::Ident(..))) => true,
		_ => false,
	}
}

fn is_brackets(t:Option<&&ast::TokenTree>) -> bool {
	match t {
		Some(&&TtDelimited(..)) => true,
		_ => false,
	}
}

fn is_int(t:Option<&&ast::TokenTree>) -> bool {
	match t {
		Some(&&TtToken(_, Token::Literal(Lit::Integer(..), _))) => true,
		_ => false,
	}
}

fn is_token(t:&Token, u:&Token) -> bool {
	t == u
}

pub struct Reader<'a> {
	iter: Peekable<&'a ast::TokenTree, Items<'a, ast::TokenTree>>,
}

impl<'a> Reader<'a> {
	pub fn new<'b>(tokens:&'b [ast::TokenTree]) -> Reader<'b> {
		Reader {
			iter: tokens.iter().peekable()
		}
	}

	pub fn is_done(&mut self) -> bool {
		self.iter.is_empty()
	}

	pub fn read_comment(&mut self) -> Result<String, String> {
		match self.iter.peek() {
			Some(&&TtToken(_, Token::DocComment(value))) => {
				self.iter.next();
				Ok(value.to_string())
			},
			_ => {
				Err(format!("expecting comment"))
			}
		}
	}

	pub fn has_ident(&mut self) -> bool {
		is_ident(self.iter.peek())
	}

	pub fn read_ident(&mut self) -> Result<String, String> {
		match self.iter.peek() {
			Some(&&TtToken(_, Token::Ident(value, kind))) => {
				self.iter.next();
				Ok(value.as_str().to_string())
			},
			_ => {
				Err(format!("expecting identifier"))
			}
		}
	}

	pub fn read_ident_match(&mut self, select:&[&str]) -> Result<String, String> {
		match self.iter.peek() {
			Some(&&TtToken(_, Token::Ident(value, kind))) => {
				match select.iter().position(|p| &value.as_str().as_slice() == p) {
					None => Err(format!("unexpected identifier {}", value)),
					_ => {
						self.iter.next();
						Ok(value.as_str().to_string())
					}
				}
			},
			_ => {
				Err(format!("expecting identifier"))
			}
		}
	}

	pub fn has_brackets(&mut self) -> bool {
		is_brackets(self.iter.peek())
	}

	pub fn read_brackets(&mut self) -> Result<Rc<Delimited>, String> {
		match self.iter.peek() {
			Some(&&TtDelimited(_, ref treeptr)) => {
				self.iter.next();
				Ok(treeptr.clone())
			},
			_ => {
				Err(format!("expecting brackets"))
			}
		}
	}

	pub fn has_int(&mut self) -> bool {
		is_int(self.iter.peek())
	}

	pub fn read_int(&mut self) -> Result<int, String> {
		match self.iter.peek() {
			Some(&&TtToken(_, Token::Literal(Lit::Integer(value), kind))) => {
				self.iter.next();
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

	pub fn read_token(&mut self, t:Token) -> Result<(), String> {
		match self.iter.peek() {
			Some(&&TtToken(_, ref u)) => {
				if is_token(&t, u) {
					self.iter.next();
					return Ok(())
				}
			}
			_ => {}
		}
		Err(format!("expecting simple token {}, got {}", t, self.iter.peek()))
	}
}

pub trait TreeUnpack {
	fn unpack_tree(&self) -> Reader;
}

impl TreeUnpack for Rc<Delimited> {
	fn unpack_tree(&self) -> Reader {
		Reader::new(self.tts.as_slice())
	}
}
