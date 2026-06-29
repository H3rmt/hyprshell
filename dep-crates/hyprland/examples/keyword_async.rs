/// Demostrates how to fetch and set keywords asyncronously
///
/// Usage: cargo run --example keyword_async <keyword> <value>
/// Example: cargo run --example keyword_async decoration:rounding (prints value)
/// Example: cargo run --example keyword_async decoration:rounding  15 (sets value)
use hyprland::keyword::Keyword;

#[tokio::main(flavor = "current_thread")]
async fn main() -> hyprland::Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let keyword = args[0].clone();

    match args.len() {
        0 => panic!("You need to pass a keyword"),
        1 => println!(
            "{} value is {}",
            keyword,
            Keyword::get_async(&keyword).await?.value
        ),
        2 => Keyword::set_async(keyword, args[1].clone()).await?,
        _ => panic!("Takes up to 2 arguments only!"),
    }
    Ok(())
}
