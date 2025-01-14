pub mod ast;
pub mod lexer;
pub mod tokens;

pub mod mdx_ast;
pub mod mdx_lexer;
pub mod mdx_tokens;

use lalrpop_util::lalrpop_mod;

lalrpop_mod!(pub grammar);
lalrpop_mod!(pub mdx_grammar);

#[cfg(test)]
mod test;
