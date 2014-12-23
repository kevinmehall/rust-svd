//! Peripheral macro.

#![feature(phase, plugin_registrar, macro_rules, quote)]
#![feature(globs)]

#![allow(unused_variables)]
#![allow(dead_code)]

extern crate syntax;
extern crate rustc;

use syntax::ptr::P;
use syntax::codemap;
use syntax::ext::base::{ExtCtxt, MacResult, MacItems};
use syntax::util::small_vector::SmallVector;
use syntax::parse::token::Token;

use ast::PeripheralAst;
use reader::Reader;
use parser::*;
use generator::*;

pub mod ast;
pub mod reader;
pub mod parser;
pub mod generator;

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut rustc::plugin::Registry) {
    reg.register_macro("foo", regs_macro);
}

fn generate_peripherals<'a>(cx: &'a mut ExtCtxt, sp: codemap::Span, reader:&mut Reader<'a>)
  -> Result<Vec<P<syntax::ast::Item>>, String>
{
	let mut result = vec![];

    let mut systems:Vec<String> = vec![];
    while let Ok(..) = reader.read_ident_match(&["system"]) {
        systems.push(try!(reader.read_ident()));
        try!(reader.read_token(Token::Semi));
    }

    let mut peripherals:Vec<PeripheralAst> = vec![];
    while !reader.is_done() {
	    match parse_peripheral(reader) {
	    	Err(err) => {
	    		cx.span_err(sp, err.as_slice());
	    		break;
	    	},
	    	Ok(peripheral) => {
                peripherals.push(peripheral);
	    	}
	    }
	}

    for p in peripherals.iter() {
        let tokens = generate_peripheral(cx, p);
        result.push(quote_item!(cx, $tokens).unwrap());
    }

    // Generate systems.
    for sys in systems.into_iter() {
        result.push_all(generate_system(cx, sys, &peripherals).as_slice());
    }

	if !reader.is_done() {
    	Err(format!("unexpected content at end of macro"))
	} else {
		Ok(result)
	}
}

fn regs_macro<'a>(cx: &'a mut ExtCtxt, sp: codemap::Span, tokens: &[syntax::ast::TokenTree])
  -> Box<MacResult + 'a>
{
    // for i in tokens.iter() {
    // 	println!("{}", i);
    // }

    let results = match generate_peripherals(cx, sp, &mut Reader::new(tokens)) {
    	Ok(results) => results,
    	Err(err) => {
    		cx.span_err(sp, err.as_slice());
            vec![]
    	}
    };
    MacItems::new(SmallVector::many(results).into_iter())
}
