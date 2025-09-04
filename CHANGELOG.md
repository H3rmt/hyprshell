# Changelog

## [4.6.0](https://github.com/H3rmt/hyprshell/compare/v4.5.0...v4.6.0) (2025-09-04)


### Features

* added shell completions ([a74fa47](https://github.com/H3rmt/hyprshell/commit/a74fa4777a2f905b6a0f0269401c564e60068692))
* added toml to ron migration (toml dropped, as it can't store None values) ([9d1e370](https://github.com/H3rmt/hyprshell/commit/9d1e370b7c2223171a6fb92d0aba0f9cc2a9ca01))
* better config migrations (allow multi version migrations) ([299d388](https://github.com/H3rmt/hyprshell/commit/299d38816d02dadd97160e7a13088f6aaca2d4ea))
* enhance ini parsing and added new cli command to get, list and set default apps ([2036a3c](https://github.com/H3rmt/hyprshell/commit/2036a3cb1a3588b97eaea5967e8900cff73726c8))
* show info when new version detected ([e80fe65](https://github.com/H3rmt/hyprshell/commit/e80fe65a86da07448936babfd4a59ee33340217f))


### Bug Fixes

* apply user style with user priority ([b700ee0](https://github.com/H3rmt/hyprshell/commit/b700ee0e4954b9e463ba64263953dacfc36ad097))
* close overview with open key ([aa5be3f](https://github.com/H3rmt/hyprshell/commit/aa5be3f560bd1fd7bf0026d8c9e09b3f4b4d15b4))
* **deps:** update rust crate anyhow to v1.0.99 ([a7e96f3](https://github.com/H3rmt/hyprshell/commit/a7e96f388deb7c132bf5bd2f35e88be3f7d56d45))
* **deps:** update rust crate notify to v8.1.0 ([611cde3](https://github.com/H3rmt/hyprshell/commit/611cde3cb681f030a038439f321421d4e875222e))
* enable show_actions_submenu for nix users ([8c19498](https://github.com/H3rmt/hyprshell/commit/8c1949892c46da3ff24548ccd904741694339fb0))
* exclude empty workspaces in switch mode ([10786eb](https://github.com/H3rmt/hyprshell/commit/10786eb5001f1ed7abc3bb2483741b30e410152f))
* exit app when removing / adding monitors ([032a047](https://github.com/H3rmt/hyprshell/commit/032a047597cab6f2dbe0c6e4482c8d05eb7fbdca))
* fix cargo install cargo-workspaces ([e62a334](https://github.com/H3rmt/hyprshell/commit/e62a33465633ae87653fed679549b4cfc988f73b))
* fix cargo ws publish, allow buildscript to run make ([14b5b5a](https://github.com/H3rmt/hyprshell/commit/14b5b5a2ab9e6501c2c125203f44808a80314649))
* fix missing version in dependency of custom hyprland-rs ([94747cc](https://github.com/H3rmt/hyprshell/commit/94747cc81f554faf034bb5b7c5ec04dcbad03119))
* fix publish workflow check commit ([29e440f](https://github.com/H3rmt/hyprshell/commit/29e440f6f0685e93d24291196b16623d7721bcd6))
* fixed select window in overview ([71080a9](https://github.com/H3rmt/hyprshell/commit/71080a9211bd63de4aa0c1405810dfa1126c180c))
* **nix-flake:** update flake.lock ([918e40b](https://github.com/H3rmt/hyprshell/commit/918e40beb8e70649e52ffcf8dd21747bcdc3f27f))
* **nix-flake:** update flake.lock ([4eeaa57](https://github.com/H3rmt/hyprshell/commit/4eeaa5710aa7503b9a1307c8016879fd8df664ec))
* **plugin:** fix open overview after mouse button press ([bd03613](https://github.com/H3rmt/hyprshell/commit/bd0361332ca313d527daec85d7cfdc0d057a5fc1))
* reload desktop files, etc. after opening launcher ([a679985](https://github.com/H3rmt/hyprshell/commit/a67998550abd8069c47a6d2bd762b72208a70b2f))
* style changes, liquid gras css updated ([6871827](https://github.com/H3rmt/hyprshell/commit/687182774094aa572441ece0ad9b44c52612a196))
* typos in home manager configuration ([a9fc51e](https://github.com/H3rmt/hyprshell/commit/a9fc51e6e8d1e23302e17cc905f2f8285744c9fc))
* use bash to start apps ([c86dee1](https://github.com/H3rmt/hyprshell/commit/c86dee15733bc86f0ab81cceeb84bb1671876da3))
* use new hyprland-rs Instance ([c519605](https://github.com/H3rmt/hyprshell/commit/c51960581592469653380adfc03d3ea2f78e2e3a))
* use toml extension on lookup config file ([0ab9e9d](https://github.com/H3rmt/hyprshell/commit/0ab9e9dee21e70e80a815dbea7d833a32f5497cc))


### Code Refactoring

* add hyprland plugin ([1412e7a](https://github.com/H3rmt/hyprshell/commit/1412e7a46a3945d7a2f76dd1d1c5ac675160a17e))
* add hyprland plugin ([33ced1c](https://github.com/H3rmt/hyprshell/commit/33ced1cab64d2864de3f3b3d41f53484ec9adedb))
* better animations for launcher ([9c2a71c](https://github.com/H3rmt/hyprshell/commit/9c2a71cb275516adbd98ef084a9f5dcd564dedb2))
* build plugin at runtime ([400a93b](https://github.com/H3rmt/hyprshell/commit/400a93bbc340f933d473c393f4d71fcf2b5339ad))
* check if set desktop file is valid ([b7cfa98](https://github.com/H3rmt/hyprshell/commit/b7cfa982a371300bf523e12dbfd06f2734bc7ebd))
* fix nix wrap program ([5895432](https://github.com/H3rmt/hyprshell/commit/58954329f521daabaf67cb0c67a3f130ef9f26cc))
* implement plugin for switch mode ([9e1193e](https://github.com/H3rmt/hyprshell/commit/9e1193e14b7335af4c017593b2bcafb0a1882f90))
* more strict clippy rules ([fea4993](https://github.com/H3rmt/hyprshell/commit/fea4993df001461e1e0cc7ba64ccedfda605bb3c))
* return Ok / Err from socket ([61e09b7](https://github.com/H3rmt/hyprshell/commit/61e09b799860451290d68f46a0b98dc279ae2962))
* separate config crate ([9d1e370](https://github.com/H3rmt/hyprshell/commit/9d1e370b7c2223171a6fb92d0aba0f9cc2a9ca01))
* split launcher plugin into 2 data ([c0ff0b7](https://github.com/H3rmt/hyprshell/commit/c0ff0b74cce58f0eec04f79754971863770ebac3))
* store clippy lints in cargo.toml ([4387f52](https://github.com/H3rmt/hyprshell/commit/4387f52881a05780efa34de0457271363400c121))
* use different dirs for debug mode ([045def3](https://github.com/H3rmt/hyprshell/commit/045def381c611950851d4819b973a791ea896a2c))
* use global desktopfile and mime cache ([ea64c40](https://github.com/H3rmt/hyprshell/commit/ea64c408bbb4ca1d84eabadab3d6a9db9632b1de))
* use keymaps in hyprland plugin ([e0cd4da](https://github.com/H3rmt/hyprshell/commit/e0cd4daae2ed88ab7129a1f2c4dffca98d78d371))
* use make to improve plugin build time ([df032f4](https://github.com/H3rmt/hyprshell/commit/df032f4a1559dd13e60131a731e4d6e8d449de98))
* using plugin for all keyboard interactions ([ae39988](https://github.com/H3rmt/hyprshell/commit/ae39988bc3206089fb1b729d98b1ead4df1f32b9))
* using plugin for all keyboard interactions ([9dfa549](https://github.com/H3rmt/hyprshell/commit/9dfa5494bfd526d3acace3730f0edd9f7fbbe1eb))


### Documentation

* update CONFIGURE.md ([#304](https://github.com/H3rmt/hyprshell/issues/304)) ([9e590a0](https://github.com/H3rmt/hyprshell/commit/9e590a0339b547dfceea07ee8165eda649b1c8ec))
* updated docs ([d713230](https://github.com/H3rmt/hyprshell/commit/d713230fe5435b1f75e78b53c8e749423283a8af))

## [4.5.0](https://github.com/H3rmt/hyprshell/compare/v4.4.3...v4.5.0) (2025-06-27)


### Features

* added path plugin ([910aa35](https://github.com/H3rmt/hyprshell/commit/910aa357abc27c4c6f801d19920feed4e05549f1))


### Documentation

* update screenshots ([e9b8c7c](https://github.com/H3rmt/hyprshell/commit/e9b8c7ce5b2915ec13adf2cbd8994a3eb669408f))

## [4.4.3](https://github.com/H3rmt/hyprshell/compare/v4.4.2...v4.4.3) (2025-06-26)


### Bug Fixes

* fix modifier keys to launch again... ([d31ee66](https://github.com/H3rmt/hyprshell/commit/d31ee669c8da6460b1b0821b9b66783fd10c4a0e))
* use correct keys for switch mode ([b1c3353](https://github.com/H3rmt/hyprshell/commit/b1c335325f68ca1c5810fac772072640d5db464f))


### Code Refactoring

* changed PKGBUILD ([b3f207d](https://github.com/H3rmt/hyprshell/commit/b3f207d7bc0c27892d30fa2420053f27b8e714e6))

## [4.4.2](https://github.com/H3rmt/hyprshell/compare/v4.4.1...v4.4.2) (2025-06-26)


### Bug Fixes

* fix launcher keybinds ([05b2867](https://github.com/H3rmt/hyprshell/commit/05b28670edef4ca23e47100785b90b57b8311c06))
* fix modifier keys to launch, added launch_modifier ([19ba571](https://github.com/H3rmt/hyprshell/commit/19ba57169ae4c77e1e5331c764828f7f67703b90))
* **nix-flake:** update flake.lock ([abeaea5](https://github.com/H3rmt/hyprshell/commit/abeaea56cb568cb8e30ab8289e194ce10c46ec26))
* run flake update ci on hyprshell branch ([b39d435](https://github.com/H3rmt/hyprshell/commit/b39d435af9ceb60b25667bf70965499828f1f719))


### Code Refactoring

* simplify flake ([431536c](https://github.com/H3rmt/hyprshell/commit/431536cddc88606ebe2246ddb755c10a2db51643))


### Documentation

* update nix docs ([d0f45f1](https://github.com/H3rmt/hyprshell/commit/d0f45f1fdaeac348e25d5c5f7c95f76cabefb3d0))

## [4.4.1](https://github.com/H3rmt/hyprshell/compare/v4.4.0...v4.4.1) (2025-06-24)


### Bug Fixes

* run flake update ci on hyprshell branch ([b39d435](https://github.com/H3rmt/hyprshell/commit/b39d435af9ceb60b25667bf70965499828f1f719))


### Code Refactoring

* simplify flake ([431536c](https://github.com/H3rmt/hyprshell/commit/431536cddc88606ebe2246ddb755c10a2db51643))


### Documentation

* update nix docs ([d0f45f1](https://github.com/H3rmt/hyprshell/commit/d0f45f1fdaeac348e25d5c5f7c95f76cabefb3d0))

## [4.4.0](https://github.com/H3rmt/hyprshell/compare/v4.3.1...v4.4.0) (2025-06-24)


### Features

* add tui question for switch&gt;show_workspaces ([8e0d925](https://github.com/H3rmt/hyprshell/commit/8e0d9254ec9e0556a1f7b214acbb70a98710c1ca))
* added show_workspaces flag ([bbba547](https://github.com/H3rmt/hyprshell/commit/bbba5472ed493b4ce5f0b4efe47e98c303e734b6))


### Bug Fixes

* dont allow opening overview and switch at the same time. ([7b61fd5](https://github.com/H3rmt/hyprshell/commit/7b61fd58627a7fe5be85c4322fd506b57b8685f0))
* dont launch plugin entries when typing num instead of ctrl + num ([7b61fd5](https://github.com/H3rmt/hyprshell/commit/7b61fd58627a7fe5be85c4322fd506b57b8685f0))
* fix nix strip_html_from_workspace_title ([e3f02ea](https://github.com/H3rmt/hyprshell/commit/e3f02ea902fd84fe6201fe29bf221e9804100f57))
* fix nix version setting ([06fd3f7](https://github.com/H3rmt/hyprshell/commit/06fd3f7f0de5e3f8c7ae80eb99e98f953766c81e))
* generate correct keybinds for opening overview with super + &lt;key&gt;, fix [#254](https://github.com/H3rmt/hyprshell/issues/254) ([9d52a57](https://github.com/H3rmt/hyprshell/commit/9d52a57baa17a0b41897073b3602619bc04f53d4))
* mark the current workspace as active if the overview is opened without an active client ([f6eaa02](https://github.com/H3rmt/hyprshell/commit/f6eaa0212782e35d2d0d051f77da53b3efaeda7c))
* removed old nix navigate assertions ([8151fba](https://github.com/H3rmt/hyprshell/commit/8151fba64718e69e7ed3a9d46bccfd54a84329d6))


### Code Refactoring

* add better nix checks and switch to nix only for ci ([b5f8682](https://github.com/H3rmt/hyprshell/commit/b5f86823ed599f2f133b6cba8271248417fbe03f))
* added check-if-default command for ci ([db912d1](https://github.com/H3rmt/hyprshell/commit/db912d1f1b6919e71288b9cc75e703071bce559e))
* separate nix code utils ([a3b61e8](https://github.com/H3rmt/hyprshell/commit/a3b61e869c6ff60b4b755f4b2c977ffbc4d82d91))


### Documentation

* update CONFIGURE.md ([24b9799](https://github.com/H3rmt/hyprshell/commit/24b979918340dd76515686996cf836e61cd96694))

## [4.3.1](https://github.com/H3rmt/hyprshell/compare/v4.3.0...v4.3.1) (2025-06-21)


### Bug Fixes

* repair launcher control keys ([78147fa](https://github.com/H3rmt/hyprshell/commit/78147fa354e3b961b19ce8a9a147601434d71d06))

## [4.3.0](https://github.com/H3rmt/hyprshell/compare/v4.2.12...v4.3.0) (2025-06-21)


### Features

* switch to gtk key handling ([65a0ad5](https://github.com/H3rmt/hyprshell/commit/65a0ad5f482707cab8339c3c01195ff9b5557c1a))


### Bug Fixes

* **deps:** update rust crate libc to v0.2.174 ([b6d1089](https://github.com/H3rmt/hyprshell/commit/b6d10891ce2bc1f649a6af7d62f2f7f2fa09d74b))
* fix closing on mod keys other than open key ([d6aba16](https://github.com/H3rmt/hyprshell/commit/d6aba16a2eedfee40bd74feae95750d75c2edf85))
* fix colored output for explain command ([5cbf8ed](https://github.com/H3rmt/hyprshell/commit/5cbf8ede5d3323b6d7484ab1b638f26842165e83))


### Code Refactoring

* remove launcher dependency of overview/switch crate ([df40faa](https://github.com/H3rmt/hyprshell/commit/df40faaec4bf5e5466575d6189194b97e303ac78))
* remove submaps (10/10) ([c7551ea](https://github.com/H3rmt/hyprshell/commit/c7551ea526583841b5de1b8071ee22a6d5b158fd))
* remove submaps (3/?) ([9f0c09e](https://github.com/H3rmt/hyprshell/commit/9f0c09e32d8a2ead763f38810f06b71e7dfa93e9))
* remove submaps (4/?) ([765c88c](https://github.com/H3rmt/hyprshell/commit/765c88c331c3b0e7d834857ef8dc76f235a342ad))
* remove submaps (5/?) ([65b31cf](https://github.com/H3rmt/hyprshell/commit/65b31cf95ec0edb16357cd865f8d2fa6a22f4e6a))
* remove submaps (6/?) ([0bc4396](https://github.com/H3rmt/hyprshell/commit/0bc43963a7783a03fc0681f43549e2cd55a56bc7))
* remove submaps (7/?) ([fdc797d](https://github.com/H3rmt/hyprshell/commit/fdc797d52cba759167a93f88e03b66609ef54f78))
* remove submaps (8/?) ([9de4678](https://github.com/H3rmt/hyprshell/commit/9de46780f4b2cfeb0f5f4fe9e39c67aebc9e8730))
* remove submaps (9/?) ([b3f0209](https://github.com/H3rmt/hyprshell/commit/b3f02096c69aa90ca49eebd02b922fb8a127c1b2))

## [4.2.12](https://github.com/H3rmt/hyprshell/compare/v4.2.11...v4.2.12) (2025-06-20)


### Bug Fixes

* repair ci ([eaf5391](https://github.com/H3rmt/hyprshell/commit/eaf5391a28a9821caaec626baab2a78211ee7cdd))
* repair ci ([e0f8af6](https://github.com/H3rmt/hyprshell/commit/e0f8af675a8702fa50c062030b00c51d2f0d4c30))
* repair ci ([ca782ce](https://github.com/H3rmt/hyprshell/commit/ca782ce03493b3d01f1e30540b7869ff20b0ad1e))
* repair ci ([761bd1b](https://github.com/H3rmt/hyprshell/commit/761bd1b3444f35c32924efbdd1fc375452600096))
* repair ci ([0aadfcc](https://github.com/H3rmt/hyprshell/commit/0aadfcc95262f34f49fd535fc88a35f767997423))
* show toast when using switch mode ([a34a9bb](https://github.com/H3rmt/hyprshell/commit/a34a9bbe2460dd44213fc5c15d7c34a140b19315))
* use release branch in ci to create new commits ([d8a489e](https://github.com/H3rmt/hyprshell/commit/d8a489e80070f3a3c7d9d451a9f4b04f703fb2d9))

## [4.2.11](https://github.com/H3rmt/hyprshell/compare/v4.2.10...v4.2.11) (2025-06-20)


### Bug Fixes

* repair ci ([ca782ce](https://github.com/H3rmt/hyprshell/commit/ca782ce03493b3d01f1e30540b7869ff20b0ad1e))

## [4.2.10](https://github.com/H3rmt/hyprshell/compare/v4.2.9...v4.2.10) (2025-06-20)


### Bug Fixes

* repair ci ([761bd1b](https://github.com/H3rmt/hyprshell/commit/761bd1b3444f35c32924efbdd1fc375452600096))
* repair ci ([0aadfcc](https://github.com/H3rmt/hyprshell/commit/0aadfcc95262f34f49fd535fc88a35f767997423))
* show toast when using switch mode ([a34a9bb](https://github.com/H3rmt/hyprshell/commit/a34a9bbe2460dd44213fc5c15d7c34a140b19315))
* use release branch in ci to create new commits ([d8a489e](https://github.com/H3rmt/hyprshell/commit/d8a489e80070f3a3c7d9d451a9f4b04f703fb2d9))

## [4.2.9](https://github.com/H3rmt/hyprshell/compare/v4.2.8...v4.2.9) (2025-06-20)


### Bug Fixes

* repair ci ([0aadfcc](https://github.com/H3rmt/hyprshell/commit/0aadfcc95262f34f49fd535fc88a35f767997423))

## [4.2.8](https://github.com/H3rmt/hyprshell/compare/v4.2.7...v4.2.8) (2025-06-20)


### Bug Fixes

* use release branch in ci to create new commits ([d8a489e](https://github.com/H3rmt/hyprshell/commit/d8a489e80070f3a3c7d9d451a9f4b04f703fb2d9))

## [4.2.7](https://github.com/H3rmt/hyprshell/compare/v4.2.6...v4.2.7) (2025-06-20)


### Bug Fixes

* show toast when using switch mode ([a34a9bb](https://github.com/H3rmt/hyprshell/commit/a34a9bbe2460dd44213fc5c15d7c34a140b19315))

## [4.2.6](https://github.com/H3rmt/hyprshell/compare/v4.2.5...v4.2.6) (2025-06-20)


### Bug Fixes

* show toast when using switch mode ([a34a9bb](https://github.com/H3rmt/hyprshell/commit/a34a9bbe2460dd44213fc5c15d7c34a140b19315))

## [4.2.5](https://github.com/H3rmt/hyprshell/compare/v4.2.4...v4.2.5) (2025-06-11)


### Bug Fixes

* fix run programs ([3997d2a](https://github.com/H3rmt/hyprshell/commit/3997d2a85e77d2d0e3a6799b17518ad5886aca74))

## [4.2.4](https://github.com/H3rmt/hyprswitch/compare/v4.2.3...v4.2.4) (2025-06-11)


### Bug Fixes

* diable gestures and input:follow_mouse on start and reset after close ([6a516b2](https://github.com/H3rmt/hyprswitch/commit/6a516b23e9124f577e16d2475962f8e3347237ae))

## [4.2.3](https://github.com/H3rmt/hyprswitch/compare/v4.2.2...v4.2.3) (2025-06-11)


### Bug Fixes

* fix storage of input:follow_mouse setting ([62ab0f2](https://github.com/H3rmt/hyprswitch/commit/62ab0f2f90a7235e3e2d6a8aa9267ebff4c16348))
* fix systemd exit ([26ae103](https://github.com/H3rmt/hyprswitch/commit/26ae103729696a02069c873bad76c2edbe9dcdf6))

## [4.2.2](https://github.com/H3rmt/hyprswitch/compare/v4.2.1...v4.2.2) (2025-06-11)


### Bug Fixes

* add HYPRSHELL_RELOAD_TIMEOUT to change timeout ([764cbb2](https://github.com/H3rmt/hyprswitch/commit/764cbb211ee0a2a1382443b34e58e8a4a035fdb9))

## [4.2.1](https://github.com/H3rmt/hyprswitch/compare/v4.2.0...v4.2.1) (2025-06-11)


### Bug Fixes

* add show_actions_submenu with default false ([c0828d6](https://github.com/H3rmt/hyprswitch/commit/c0828d6706157728e4af7742a8e97f7190a8eec0))
* file watchers work again ([73c7ebf](https://github.com/H3rmt/hyprswitch/commit/73c7ebf0914442c85b8af49e920fdbdd87be10a9))
* use correct path to generate config ([112e7d2](https://github.com/H3rmt/hyprswitch/commit/112e7d26acde6dd98805b69b807b6a5004763a96))

## [4.2.0](https://github.com/H3rmt/hyprswitch/compare/v4.1.1...v4.2.0) (2025-06-11)


### Features

* better window selection on empty workspace ([24b36b3](https://github.com/H3rmt/hyprswitch/commit/24b36b38cc0b127fe2de2a59c223936ffc9c988e))
* **nix:** Add `show_when_empty` ([8ebb333](https://github.com/H3rmt/hyprswitch/commit/8ebb333cd0dad0ff919ea1790136c8e6120d4560))


### Bug Fixes

* close socket after restarting app ([b4a8f2e](https://github.com/H3rmt/hyprswitch/commit/b4a8f2e61d503fb6eee7850ce1ce6f33dfab72bf))
* debounce reload ([5858cfd](https://github.com/H3rmt/hyprswitch/commit/5858cfd475f1e3b604f6d035bdc5507977833af5))
* **deps:** update rust crate clap to v4.5.40 ([19272f4](https://github.com/H3rmt/hyprswitch/commit/19272f40aec98c2820919f4d5924533f12b15fc9))
* **deps:** update rust crate toml to v0.8.23 ([6268be8](https://github.com/H3rmt/hyprswitch/commit/6268be8a74f85eb90dc1d7d94a5057ce6534ce89))
* handle sigterm and reset submap ([1516024](https://github.com/H3rmt/hyprswitch/commit/1516024a04d46f4433f52984197bf6f39eeee6a6))
* improved file watcher, file descriptor limit was reached if reloaded too many times ([b4a8f2e](https://github.com/H3rmt/hyprswitch/commit/b4a8f2e61d503fb6eee7850ce1ce6f33dfab72bf))
* selecting a client with filtering form workspace without an enabled client now selects the first valid client depending on the direction instead of a first client in the workspace ([2a72d8b](https://github.com/H3rmt/hyprswitch/commit/2a72d8ba63ff7c20412d1a9a1dcd7da861c8204b))
* toml config plugins for launcher ([5858cfd](https://github.com/H3rmt/hyprswitch/commit/5858cfd475f1e3b604f6d035bdc5507977833af5))

## [4.1.1](https://github.com/H3rmt/hyprswitch/compare/v4.1.0...v4.1.1) (2025-06-02)


### Bug Fixes

* release workflow now uses deploy keys ([e2c9b89](https://github.com/H3rmt/hyprswitch/commit/e2c9b89f10505a6617f74b3e05c4129b162029ec))

## [4.1.0](https://github.com/H3rmt/hyprswitch/compare/v4.0.4...v4.1.0) (2025-06-02)


### Features

* added kill_bind if hyprshell crashes ([5e0b0fa](https://github.com/H3rmt/hyprswitch/commit/5e0b0fa2cf8b7c7d4902fafff0b0dc4b2d03a84a))
* better parsing of desktop files(ini) to add DesktopActions in launcher ([a304809](https://github.com/H3rmt/hyprswitch/commit/a3048098b1702277cf25e0454e0a7dd3d48c61ee))
* faster open speeds by applying submaps earlier ([a304809](https://github.com/H3rmt/hyprswitch/commit/a3048098b1702277cf25e0454e0a7dd3d48c61ee))


### Bug Fixes

* use new ini parser everywhere ([8db2643](https://github.com/H3rmt/hyprswitch/commit/8db2643a7c5e5b51a316d4654268c38bc2202be4))

## [4.0.4](https://github.com/H3rmt/hyprswitch/compare/v4.0.3...v4.0.4) (2025-06-01)


### Bug Fixes

* more debugging for default browser to fix [#188](https://github.com/H3rmt/hyprswitch/issues/188) ([551bfd0](https://github.com/H3rmt/hyprswitch/commit/551bfd0b58aedbccc93236595beab63dcf9195dc))

## [4.0.3](https://github.com/H3rmt/hyprswitch/compare/v4.0.2...v4.0.3) (2025-06-01)


### Bug Fixes

* fixed icon scaling ([5a0489a](https://github.com/H3rmt/hyprswitch/commit/5a0489a39742d89abed3760096c3dccee2fa5845))
* remove launch animation from plugins after close ([685b7bd](https://github.com/H3rmt/hyprswitch/commit/685b7bdf205cafc48e9a21aaccd9428e774905d9))
* use dbus to open if no browser was found ([c074a56](https://github.com/H3rmt/hyprswitch/commit/c074a56ac7603c14754a4b20a65eeb095e2a103d))

## [4.0.2](https://github.com/H3rmt/hyprswitch/compare/v4.0.1...v4.0.2) (2025-05-31)


### Bug Fixes

* fixed the PKGBUILD for arch ([89c06ba](https://github.com/H3rmt/hyprswitch/commit/89c06baa318157827e7042adeaa4ca274b251756))

## [4.0.1](https://github.com/H3rmt/hyprswitch/compare/v4.0.0...v4.0.1) (2025-05-31)


### Bug Fixes

* added the PKGBUILD for arch ([93ca69b](https://github.com/H3rmt/hyprswitch/commit/93ca69b15061f4ad8e4f1bcb674dba59c278571b))

## [4.0.0](https://github.com/H3rmt/hyprswitch/compare/v0.8.2...v4.0.0) (2025-05-31)


### Features

* add animation to plugin close launch ([b944274](https://github.com/H3rmt/hyprswitch/commit/b9442742b5a357966061c66e594b9104d158fe7b))
* add calc plugin ([06c8a41](https://github.com/H3rmt/hyprswitch/commit/06c8a41db42d482b777f158fcf9eb1b23708cc13))
* add debug command ([00afa1e](https://github.com/H3rmt/hyprswitch/commit/00afa1e34b9716a041d3bd33734700836075bd70))
* Add NixOS `home-manager` module ([cd20717](https://github.com/H3rmt/hyprswitch/commit/cd207178a0cfd44c7ad1069880ef35532f5547ae))
* add run shell commands from launcher ([879cbba](https://github.com/H3rmt/hyprswitch/commit/879cbba0597281c14b07dadfe03149b4323caf6f))
* add websearch plugin ([1a079d1](https://github.com/H3rmt/hyprswitch/commit/1a079d1552036f8713ba69f33271475ff4a41103))
* added `data` command to see LaunchHistory ([8e3de53](https://github.com/H3rmt/hyprswitch/commit/8e3de53b31834c8a034d28d26d72ebcbbd4d9815))
* added click on clients and workspaces in overview and switch ([328fc3b](https://github.com/H3rmt/hyprswitch/commit/328fc3b432b28ad1e390be10f945e1425708d430))
* added config file migrations ([db2f6cd](https://github.com/H3rmt/hyprswitch/commit/db2f6cd9fb3c08ab2f9858fb9c1dac61540353b5))
* added custom args for hyprshell systemd ([aa01139](https://github.com/H3rmt/hyprswitch/commit/aa01139aebfe2dcd717b670ac6ce557f93c2f1d0))
* added show_when_empty ([6f916d5](https://github.com/H3rmt/hyprswitch/commit/6f916d5b0355293eb0d4007b3c996deddb943c0d))
* added systemd generation (use --no-systemd to disable) ([97c2c7f](https://github.com/H3rmt/hyprswitch/commit/97c2c7f88c3ba221863a43b8adbfb50b444fa841))
* better debug commands ([b17a393](https://github.com/H3rmt/hyprswitch/commit/b17a393b04201beab8b582a340d1bb80bef5cda2))
* click on entry in launcher works ([87076ef](https://github.com/H3rmt/hyprswitch/commit/87076ef8c715fc0f7e29a7c2187aa59f17514df5))
* NixOS Support ([#171](https://github.com/H3rmt/hyprswitch/issues/171)) ([d42f12b](https://github.com/H3rmt/hyprswitch/commit/d42f12be62c08d3764bc91b034bc7aa05d531608))
* rewrite hyprswitch ([198cd0f](https://github.com/H3rmt/hyprswitch/commit/198cd0f5ae03210b46cfaba8dbf5f8d30fcc77a9))
* rewrite hyprswitch ([0d834ab](https://github.com/H3rmt/hyprswitch/commit/0d834ab64cb607d2bc10ca7b13d2642728592f50))


### Bug Fixes

* add aur publish ([fc4f9ab](https://github.com/H3rmt/hyprswitch/commit/fc4f9ab4f040646ff075930a86457bf7d4f3e77c))
* add nix back ([9efadcd](https://github.com/H3rmt/hyprswitch/commit/9efadcdc37ae0c639ab36a1880e3d48b1b6c51a2))
* allow hold of tab and arrow keys to switch ([e0aaa57](https://github.com/H3rmt/hyprswitch/commit/e0aaa570f54fce0155631c5720805886a0c275a1))
* arrow keys in the launcher ([179ca7b](https://github.com/H3rmt/hyprswitch/commit/179ca7b45e865875584a4c658a0455ea00af0bc6))
* background loading of icons ([f5c8ff0](https://github.com/H3rmt/hyprswitch/commit/f5c8ff0e29713432726bb841bafd8dd6729331fa))
* better icon detection ([453888e](https://github.com/H3rmt/hyprswitch/commit/453888e61551946cfa3dec92409df606f7aa04db))
* check for config file extensions at start ([7a41bb2](https://github.com/H3rmt/hyprswitch/commit/7a41bb2bf8abea7b2bc2efc2544deb26243faf7c))
* ci release ([4a0f45f](https://github.com/H3rmt/hyprswitch/commit/4a0f45f466d47a12e6cab70d2e71c835287287a0))
* close launcher on esc ([a003b82](https://github.com/H3rmt/hyprswitch/commit/a003b8227b709a7bb186e2b3703ec70f3f16dd7d))
* detect socat path at runtime ([312f667](https://github.com/H3rmt/hyprswitch/commit/312f6677165a023466a115d8a61f2927b31adc71))
* don't exit launcher when no items ([a4356b7](https://github.com/H3rmt/hyprswitch/commit/a4356b783c148f61736fcd9e4af23d63b1be7c85))
* don't unset all CSS styles at the start, only necessary ([b42d25c](https://github.com/H3rmt/hyprswitch/commit/b42d25c1d342d10a5af378e1ebcae167a83ca01f))
* filter apps in the launcher by name, exec and details ([ab0f23e](https://github.com/H3rmt/hyprswitch/commit/ab0f23e43c4ce540e14fe3f0ac515fa52ec56ab9))
* fix ci publish ([bb1e659](https://github.com/H3rmt/hyprswitch/commit/bb1e65987ac7c371b29cbf09000bb4f117f5b0e9))
* fix ci release ([e498ca1](https://github.com/H3rmt/hyprswitch/commit/e498ca19f86fcf2d4668fa7582f9320232fd194d))
* fix ci release ([6dc00c7](https://github.com/H3rmt/hyprswitch/commit/6dc00c719ef7beaea70c86f7851c6de8cdf9117e))
* fix focus window problem ([3c702a1](https://github.com/H3rmt/hyprswitch/commit/3c702a1da63d56168aa6e7ba2932842c55f97c8d))
* fix multiple run week caches not being added together ([c027209](https://github.com/H3rmt/hyprswitch/commit/c027209d6f01ec10210420182ea3283380f8e74f))
* Fix Nix Build ([02bd2c4](https://github.com/H3rmt/hyprswitch/commit/02bd2c400616cb96cd21ccc6b2f530143fcb511b))
* fix overflow for selecting workspaces ([9efadcd](https://github.com/H3rmt/hyprswitch/commit/9efadcdc37ae0c639ab36a1880e3d48b1b6c51a2))
* fix overview click on client or workspace ([71d445c](https://github.com/H3rmt/hyprswitch/commit/71d445c0f310f986b769eb420b55e21e9559d94d))
* fix panic when listening for changes on nonexisting file ([2e02509](https://github.com/H3rmt/hyprswitch/commit/2e025099b4b259844a6f5192a4e9db3db10a00ba))
* fix right alt bind ([4394240](https://github.com/H3rmt/hyprswitch/commit/43942408b0febe18026e278d2e4cffd7eece25db))
* fix slow start times ([0c4bb95](https://github.com/H3rmt/hyprswitch/commit/0c4bb9585806de61f73e885026083e49b2fe2048))
* fix switch mode monitor select ([b8c1d44](https://github.com/H3rmt/hyprswitch/commit/b8c1d440d47fe4a6ca2c1690e1b0be65b81de1df))
* fix versions ([899b2cc](https://github.com/H3rmt/hyprswitch/commit/899b2cc95018a342c2bf45d0fdf0ef971616be7b))
* fix versions ([e9565fd](https://github.com/H3rmt/hyprswitch/commit/e9565fd627c85ed4fa8fcf26cd35c06ceeaeb2b7))
* fix versions ([8d9a5ee](https://github.com/H3rmt/hyprswitch/commit/8d9a5eeeee5d74fef9501fec39c5d8692061ec97))
* fix versions ([05f3da6](https://github.com/H3rmt/hyprswitch/commit/05f3da60197e59d6a882fa0282127ae62091f879))
* fix versions ([b4e0380](https://github.com/H3rmt/hyprswitch/commit/b4e0380cced9b3ae35e3d8368fb336edc5530274))
* fix versions ([14e4a72](https://github.com/H3rmt/hyprswitch/commit/14e4a725dfe6b0d70c235db80069238338ec2890))
* fix versions ([89711a6](https://github.com/H3rmt/hyprswitch/commit/89711a628c45d6286742ac274ad4ec2fe487faed))
* fix wrong name in hm module ([99386d5](https://github.com/H3rmt/hyprswitch/commit/99386d56c0e40e65bc491e2b87cf28ef83305f6d))
* force command now accepts args ([7a41bb2](https://github.com/H3rmt/hyprswitch/commit/7a41bb2bf8abea7b2bc2efc2544deb26243faf7c))
* get socat path at buildtime ([a4356b7](https://github.com/H3rmt/hyprswitch/commit/a4356b783c148f61736fcd9e4af23d63b1be7c85))
* make css optional ([b0c36ee](https://github.com/H3rmt/hyprswitch/commit/b0c36eeb3f9dfbe9a85933505dc618cd0c231308))
* move scripts ([97329b7](https://github.com/H3rmt/hyprswitch/commit/97329b723d4c90e674fa74d65fb4397252266b85))
* moved size_factor to scale for a more sensible default and bounds check ([a85f3b8](https://github.com/H3rmt/hyprswitch/commit/a85f3b8948211711acf232b9834f4c9c2afadf61))
* nix allow source and text attributes ([dcd9e41](https://github.com/H3rmt/hyprswitch/commit/dcd9e417071e80cfd489eaefc8908a6b12a324eb))
* nix json config fixes ([ced4a45](https://github.com/H3rmt/hyprswitch/commit/ced4a45ef6b8361663b30062298f3084f2219a37))
* **nix:** Fix HM Module ([b8367b4](https://github.com/H3rmt/hyprswitch/commit/b8367b4de9dcd2a5b6e49a3e9322534657de4886))
* open windows earlier ([b42d25c](https://github.com/H3rmt/hyprswitch/commit/b42d25c1d342d10a5af378e1ebcae167a83ca01f))
* reload desktop maps on close ([204358d](https://github.com/H3rmt/hyprswitch/commit/204358dc20ad49ff44d79444f707d20a4535d0da))
* remove size_factor from config ([77b53bd](https://github.com/H3rmt/hyprswitch/commit/77b53bd205d32c0619d244a601cea8304f4a4b9c))
* remove socat dependency ([01758a6](https://github.com/H3rmt/hyprswitch/commit/01758a6d6384c5ad73a841c3f2e8b90ee9912393))
* search for installed terminals from PATH ([453888e](https://github.com/H3rmt/hyprswitch/commit/453888e61551946cfa3dec92409df606f7aa04db))
* show recent windows on one screen only ([f6a3016](https://github.com/H3rmt/hyprswitch/commit/f6a301689827ebb27afc10d445715c070a5762f1))
* some focus fixes ([9fa8009](https://github.com/H3rmt/hyprswitch/commit/9fa80093f943f6941aeaeb424a262cb3b4c40ec6))
* sort launcher applications by shorted exec instead of full (removed /bin/flatpak...) ([b17a393](https://github.com/H3rmt/hyprswitch/commit/b17a393b04201beab8b582a340d1bb80bef5cda2))
* speedup animation ([179ca7b](https://github.com/H3rmt/hyprswitch/commit/179ca7b45e865875584a4c658a0455ea00af0bc6))
* switch window if no results are present in launcher ([50a81a0](https://github.com/H3rmt/hyprswitch/commit/50a81a0503f8f259374fdafab73e383f930902fe))
* try all config extensions if file missing ([6cd4799](https://github.com/H3rmt/hyprswitch/commit/6cd4799198c7b170537135a611c8ed88b97aa62f))
* try to fix publication to creates.io ([c911a07](https://github.com/H3rmt/hyprswitch/commit/c911a078e40655fd869df317479a6a93cce508b2))
* update documentation ([229921d](https://github.com/H3rmt/hyprswitch/commit/229921d82167d59670cdeab488677b372ecafe73))
* Update Nix Package ([a147d38](https://github.com/H3rmt/hyprswitch/commit/a147d385ee5928e0616d232f741657069242272f))
* use char instead of String for key for websearch plugins ([eba4282](https://github.com/H3rmt/hyprswitch/commit/eba42823569cbf19dcb35cf37cc78db7bcdb0e3b))


### Documentation

* fix css explain images ([d86d566](https://github.com/H3rmt/hyprswitch/commit/d86d5667201031915100391c1eeb9571e763f370))


### Continuous Integration

* fix ci release ([a24657b](https://github.com/H3rmt/hyprswitch/commit/a24657bb9e237e69ff2f8687577114c25de921fe))
* fix ci release ([e02df4a](https://github.com/H3rmt/hyprswitch/commit/e02df4a193718664762ba8d7e22c63814f061de3))
* fix ci release ([d8d481d](https://github.com/H3rmt/hyprswitch/commit/d8d481d672809a4f8907a156eed1705caa27a9aa))
* fix release-please again ([824bf03](https://github.com/H3rmt/hyprswitch/commit/824bf032ab3b131121b16d9c16ce9f6a5215c580))
* fix release-please again ([bce2335](https://github.com/H3rmt/hyprswitch/commit/bce23355cd3f477e95179acadd5d1401544b1822))
* fix releases ([06ad9fc](https://github.com/H3rmt/hyprswitch/commit/06ad9fc8cc85a4a6fe3584510bce0efbd1aa1425))
* switch CI back to normal repo ([9aec89c](https://github.com/H3rmt/hyprswitch/commit/9aec89c4705d1c9683d30274aa442512e3665493))
