use super::Config;
use crate::{Switch, check_switch_keys};

fn configured() -> Config {
    let switch = Switch {
        key: "F8".into(),
        keys: Some(vec!["Tab".into(), "F6".into()]),
        reverse_keys: Vec::new(),
        ..Default::default()
    };
    let windows = crate::Windows {
        switch: Some(switch),
        ..Default::default()
    };
    crate::Config {
        windows: Some(windows),
    }
    .into()
}

#[test]
fn legacy_key_and_reverse_defaults_are_unchanged() {
    let switch = Switch::default();
    assert_eq!(switch.forward_keys(), [Box::<str>::from("Tab")]);
    assert_eq!(switch.reverse_keys, [Box::<str>::from("grave")]);
    assert!(switch.keys.is_none());
    #[cfg(not(feature = "ci_no_default_config_values"))]
    {
        let serialized: super::Switch =
            serde_json::from_str(r#"{"key":"F6","modifier":"ctrl"}"#).expect("legacy config");
        let legacy: Switch = serialized.try_into().expect("runtime config");
        assert_eq!(legacy.forward_keys(), [Box::<str>::from("F6")]);
        assert_eq!(legacy.reverse_keys, [Box::<str>::from("grave")]);
    }
}

#[test]
fn explicit_lists_override_key_and_round_trip_in_every_format() {
    for reverse in [Vec::new(), vec![Box::from("grave"), Box::from("F7")]] {
        let mut original = configured();
        original
            .windows
            .as_mut()
            .expect("windows")
            .switch
            .as_mut()
            .expect("switch")
            .reverse_keys = reverse.clone();
        let json = serde_json::to_string(&original).expect("JSON serialization");
        assert_eq!(
            serde_json::from_str::<Config>(&json).expect("JSON parsing"),
            original
        );
        let ron = ron::to_string(&original).expect("RON serialization");
        assert_eq!(
            ron::from_str::<Config>(&ron).expect("RON parsing"),
            original
        );
        let toml = toml::to_string(&original).expect("TOML serialization");
        assert_eq!(
            toml::from_str::<Config>(&toml).expect("TOML parsing"),
            original
        );
        #[cfg(feature = "json5_config")]
        assert_eq!(
            serde_json5::from_str::<Config>(&json).expect("JSON5 parsing"),
            original
        );
        let runtime: crate::Config = original.clone().try_into().expect("runtime conversion");
        let switch = runtime
            .windows
            .as_ref()
            .expect("windows")
            .switch
            .as_ref()
            .expect("switch");
        assert_eq!(
            switch.forward_keys(),
            [Box::<str>::from("Tab"), Box::<str>::from("F6")]
        );
        assert_eq!(switch.reverse_keys, reverse);
        assert_eq!(Config::from(runtime), original);
    }
}

#[test]
fn invalid_lists_are_rejected() {
    for keys in [
        vec![],
        vec!["Tab", "Tab"],
        vec!["Prior", "Page_Up"],
        vec!["not_a_real_keysym"],
        vec![""],
        vec!["Tab\0"],
        vec!["CTRL+Tab"],
    ] {
        let switch = Switch {
            keys: Some(keys.into_iter().map(Box::from).collect()),
            ..Default::default()
        };
        assert!(
            check_switch_keys(&switch).is_err(),
            "accepted {:?}",
            switch.keys
        );
    }
    for reverse in [
        vec!["grave", "grave"],
        vec!["Tab"],
        vec!["not_a_real_keysym"],
    ] {
        let switch = Switch {
            reverse_keys: reverse.into_iter().map(Box::from).collect(),
            ..Default::default()
        };
        assert!(check_switch_keys(&switch).is_err());
    }
}

#[test]
fn empty_reverse_list_and_multiple_keys_are_valid() {
    let runtime: crate::Config = configured().try_into().expect("config");
    check_switch_keys(
        runtime
            .windows
            .as_ref()
            .expect("windows")
            .switch
            .as_ref()
            .expect("switch"),
    )
    .expect("valid lists");
}

#[test]
fn second_switch_uses_the_same_validation() {
    let windows = crate::Windows {
        switch_2: Some(Switch {
            keys: Some(Vec::new()),
            ..Default::default()
        }),
        ..Default::default()
    };
    assert!(
        crate::check(&crate::Config {
            windows: Some(windows)
        })
        .is_err()
    );
}

#[test]
fn explanation_does_not_claim_disabled_grave_binding() {
    let runtime: crate::Config = configured().try_into().expect("config");
    let explanation = crate::explain(&runtime, None, false);
    assert!(explanation.contains("tab / f6"));
    assert!(!explanation.contains("grave"));
}
