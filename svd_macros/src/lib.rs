#![feature(globs)]
#![feature(macro_rules)]
#![feature(phase)]
#![feature(plugin_registrar, quote)]

extern crate syntax;
extern crate rustc;
extern crate serialize;
extern crate xml;
#[phase(plugin, link)] extern crate fromxml;

pub mod common;
pub mod ast;
pub mod reader;
pub mod parser;
pub mod generator;
#[macro_escape] pub mod svd_macro;
#[macro_escape] pub mod include_macro;

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut rustc::plugin::Registry) {
    reg.register_macro("include_svd", ::include_macro::include_macro);
    reg.register_macro("svd", ::svd_macro::svd_macro);
}
