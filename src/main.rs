pub mod ast;

use lalrpop_util::lalrpop_mod;

lalrpop_mod!(grammar);

fn main() {
    println!("What?");
}
