/// Demonstrates usage of various asyncronous dispatch calls
///
/// Usage: cargo run --example dispatch_async
use hyprland::dispatch;
use hyprland::dispatch::DispatchType::*;
use hyprland::dispatch::{Corner, Dispatch, FullscreenType, WorkspaceIdentifierWithSpecial};

fn describe(desc: &str) {
    std::thread::sleep(std::time::Duration::from_secs(2));
    println!("{desc}");
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> hyprland::Result<()> {
    let program = std::env::args().skip(1).collect::<Vec<_>>().join(" ");

    println!("Executing {program}");
    dispatch!(async; Exec, &program).await?;

    describe("Moving cursor to top left");
    dispatch!(async; MoveCursorToCorner, Corner::TopLeft).await?;

    describe("Moving cursor to top right");
    dispatch!(async; MoveCursorToCorner, Corner::TopRight).await?;

    describe("Moving cursor to bottom right");
    dispatch!(async; MoveCursorToCorner, Corner::BottomRight).await?;

    describe("Moving cursor to bottom left");
    dispatch!(async; MoveCursorToCorner, Corner::BottomLeft).await?;

    describe("Moving window to next workspace");
    dispatch!(async; MoveToWorkspace, WorkspaceIdentifierWithSpecial::Relative(1), None).await?;

    describe("Moving window to previous workspace");
    dispatch!(async; MoveToWorkspace, WorkspaceIdentifierWithSpecial::Relative(-1), None).await?;

    describe("Toggling fullscreen");
    dispatch!(async; ToggleFullscreen, FullscreenType::Maximize).await?;
    describe("Reverting fullscreen");
    dispatch!(async; ToggleFullscreen, FullscreenType::Maximize).await?;

    describe("Toggling floating window");
    dispatch!(async; ToggleFloating, None).await?;
    describe("Reverting floating window");
    Dispatch::call_async(ToggleFloating(None)).await?;

    describe("Toggling split layout");
    dispatch!(async; ToggleSplit).await?;
    describe("Reverting split layout");
    Dispatch::call_async(ToggleSplit).await?;

    describe("Toggling opaque");
    dispatch!(async; ToggleOpaque).await?;
    describe("Reverting opaque");
    Dispatch::call_async(ToggleOpaque).await?;

    describe("Closing window");
    dispatch!(async; KillActiveWindow).await?;

    Ok(())
}
