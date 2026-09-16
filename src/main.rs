pub mod ast;

use lalrpop_util::lalrpop_mod;

use crate::grammar::ExprParser;

lalrpop_mod!(grammar);

fn main() -> anyhow::Result<()> {
    // let ast = grammar::ExprParser::new().parse("!1+2**2 && true")?;
    // dbg!(ast);

    let ast = grammar::StmtParser::new().parse("var magic: Fn(int) ->int");
    dbg!(ast);

    Ok(())
}
