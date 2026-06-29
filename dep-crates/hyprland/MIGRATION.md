# Migration Guide

## `0.2.5` ⟶ `0.3.0`
> This release is action packed so be prepared!

Steps (This order is recommended so lsp doesn't scream at you lol)
1. Add the prelude! This new release includes traits! `use hyprland::prelude::*;`
2. Switch to the `HResult` Result type (`hyprland::shared::HResult`)
3. [Update your data fetcher functions to structs](#update-data-fetchers)
4. [Update your dispatch functions to structs](#update-dispatchers)
5. Switch all `String`s to `&str`s in your dispatchers
6. Switch from `Positive`/`Negative` `Relative` dispatchers to just `Relative..`, and signed ints
7. When using `WorkspaceType::Special`, now include a `Some(name: String)` or `None`
8. [Update your keyword calls to new module](#update-keyword-calls)
9. [Update start listener calls](#update-start-listener-calls)
10. Follow Rustc and Clippy for everything else! 

## `0.3.*` ⟶ `0.3.3`
* Changes some integer types
* Switch to new event structs (no longer tuple!)

## `0.3.0` ⟶ `0.3.1`
* Change some types in the data
structs, because they changed (Only int types)
* Fix some dispatchers (ex. `ToggleSpecialWorkspace`)
* Update active window event, cuz I merged it with activewindowv2
* Update location of `Result` type, cuz I changed it

## `0.3.3` ⟶ `0.3.4`
* EventListener now requires mutable reference to start

## More in-depth steps

### 0.3 Update

#### Update Data fetchers
| Notation Type | Async Notation                     | Blocking Notation           |
|---------------|------------------------------------|-----------------------------|
| Old notation  | `asynchronous::get_something()`    | `blocking::get_something()` |
| New Notation  | `Something::get_async().collect()` | `Something::get().collect()`|
> The `.collect()` is required because data fetchers are now iterators!
>
> If you are just converting to a `Vec`, **USE THE `.to_vec()`** method instead,
> this being an iterator is useful if you wanna for example loop over all the workspaces


#### Update Dispatchers

| Notation Type | Async Notation           | Blocking Notation     |
|---------------|--------------------------|-----------------------|
| Old notation  | `dispatch()`             | `dispatch_blocking()` |
| New Notation  | `Dispatch::call_async()` | `Dispatch::call()`    |

#### Update start listener calls
| Notation Type | Async Notation           | Blocking Notation           |
|---------------|--------------------------|-----------------------------|
| Old notation  | `start_listener()`       | `start_listener_blocking()` |
| New Notation  | `start_listener_async()` | `start_listener()`          |

#### Update Keyword calls
This has gone through a complete rework, and now is in the `keyword` module

##### Setting a keyword
Blocking: `Keyword::set(key, val)`<br />
Asynchronous: `Keyword::set_async(key, val)`

##### Getting a keyword 
> AKA getoption

Blocking: `Keyword::get(key)`<br />
Asynchronous: `Keyword::get_async(key)`
