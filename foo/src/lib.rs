#![feature(phase, plugin_registrar, macro_rules, quote)]
#![feature(globs)]

#![allow(unused_variables)]
#![allow(dead_code)]

extern crate syntax;
extern crate rustc;

use syntax::ast;
use syntax::ptr::P;
use syntax::codemap;
use syntax::ext::base::{ExtCtxt, MacResult, MacItems};
use syntax::util::small_vector::SmallVector;

use reader::Reader;
use parser::*;
use generator::*;

mod reader;
mod parser;
mod generator;

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut rustc::plugin::Registry) {
    reg.register_macro("foo", regs_macro);
}

fn generate_peripherals<'a>(cx: &'a mut ExtCtxt, sp: codemap::Span, reader:&mut Reader<'a>)
  -> Result<Vec<P<ast::Item>>, String>
{
	let mut result = vec![];
    while !reader.iter.is_empty() {
	    match parse_peripheral(reader) {
	    	Err(err) => {
	    		cx.span_err(sp, err.as_slice());
	    		break;
	    	},
	    	Ok(peripheral) => {
	    		let tokens = generate_peripheral(cx, peripheral);
	    		result.push(quote_item!(cx, $tokens).unwrap());
	    	}
	    }
	}

	if !reader.iter.is_empty() {
    	Err(format!("unexpected content at end of macro"))
	} else {
		Ok(result)
	}
}

fn regs_macro<'a>(cx: &'a mut ExtCtxt, sp: codemap::Span, tokens: &[ast::TokenTree])
  -> Box<MacResult + 'a>
{
    // for i in tokens.iter() {
    // 	println!("{}", i);
    // }

    match generate_peripherals(cx, sp, &mut Reader::new(tokens)) {
    	Ok(results) => {
    		MacItems::new(SmallVector::many(results).into_iter())
    	},
    	Err(err) => {
    		cx.span_err(sp, err.as_slice());
    		MacItems::new(SmallVector::many(vec![]).into_iter())
    	}
    }
}
