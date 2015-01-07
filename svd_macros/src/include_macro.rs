extern crate syntax;
extern crate rustc;
extern crate serialize;
extern crate xml;

use std::borrow::ToOwned;

use syntax::ptr::P;
use syntax::codemap;
use syntax::ext::base::{ExtCtxt, MacResult, MacItems};
use syntax::util::small_vector::SmallVector;
use syntax::ast::TokenTree::TtToken;
use syntax::parse::token::Token;
use syntax::parse::token::Lit;

use common::*;
use generator;

fn parse_tokens(tokens: &[syntax::ast::TokenTree]) -> Result<(String, String), ()> {
    if tokens.len() != 3 {
        return Err(())
    }

    let svd_name = match tokens[0] {
        TtToken(_, Token::Literal(Lit::Str_(name), _)) => {
            name.as_str().to_owned()
        }
        _ => return Err(())
    };

    match tokens[1] {
        TtToken(_, Token::Comma) => {},
        _ => return Err(())
    };

    let svd_path = match tokens[2] {
        TtToken(_, Token::Literal(Lit::Str_(name), _)) => {
            name.as_str().to_owned()
        }
        _ => return Err(())
    };

    Ok((svd_name, svd_path))
}

pub fn include_macro<'a>(cx: &'a mut ExtCtxt, sp: codemap::Span, tokens: &[syntax::ast::TokenTree])
  -> Box<MacResult + 'a>
{
	let _ = (sp, tokens);

    let (svd_name, svd_path) = match parse_tokens(tokens) {
        Ok(val) => val,
        Err(()) => {
            panic!("Expected macro format include_svd!(name, path)");
        }
    };
	
    let device = load_file(svd_path.as_slice());
    let peripheral_asts = parse_xml(device);

    // Generate peripheral structs.
    let mut result:Vec<P<syntax::ast::Item>> = vec![];
    for past in peripheral_asts.iter() {
        let tokens = generator::generate_peripheral(cx, past);
        result.push(quote_item!(cx, $tokens).unwrap());
    }

    // Generate systems.
    result.push_all(generator::generate_system(cx, svd_name, &peripheral_asts).as_slice());

    MacItems::new(SmallVector::many(result).into_iter())
}
