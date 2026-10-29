# OM9-L3-04-01-042 API

- `om9_app::transform_menu::BINDINGS: &[TransformMenuBinding; 41]` (`command_line` is the typed name, `command_key` the authored menu/icon key)
- `om9_app::transform_menu::by_feature(feature: &str) -> Option<&'static TransformMenuBinding>`
- `om9_app::transform_menu::by_command(name: &str) -> Option<&'static TransformMenuBinding>`
- `om9_app::transform_menu::by_label(name: &str) -> Option<&'static TransformMenuBinding>`
- `om9_app::transform_menu::unavailable_reason(feature: &str) -> Option<&'static str>`
- `Controller::availability(feature: &str) -> Result<()>`

`BindingState` is `Implemented`, `SessionMissing` or `DeferredM08`; only `Implemented`
answers `Ok(())`.
