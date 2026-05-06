/// Demostrates how to fetch and set keywords
///
/// Usage: cargo run --example keyword <keyword> <value>?
/// Example: cargo run --example keyword decoration:rounding (prints value)
/// Example: cargo run --example keyword decoration:rounding  15 (sets value)
use hyprland::keyword::Keyword;

fn main() -> hyprland::Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let keyword = args[0].clone();

    match args.len() {
        0 => panic!("You need to pass a keyword"),
        1 => println!("{} value is {}", keyword, Keyword::get(&keyword)?.value),
        2 => Keyword::set(keyword, args[1].clone())?,
        _ => panic!("Takes up to 2 arguments only!"),
    }
    Ok(())
}
