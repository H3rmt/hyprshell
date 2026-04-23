use crate::plugins::SortableLaunchOption;
use core_lib::WarnWithDetails;
use core_lib::transfer::{Identifier, PluginName};
use rink_core::output::{NumberParts, QueryReply};
use rink_core::types::BaseUnit;
use std::path::Path;
use std::sync::{OnceLock, RwLock};
use tracing::{debug, trace};

fn get_context() -> Option<&'static RwLock<rink_core::Context>> {
    static MAP_LOCK: OnceLock<Option<RwLock<rink_core::Context>>> = OnceLock::new();
    MAP_LOCK
        .get_or_init(|| {
            rink_core::simple_context()
                .warn_details("unable to create calc context")
                .map(RwLock::new)
        })
        .as_ref()
}

pub fn init_context() {
    get_context();
}

pub fn get_calc_options(matches: &mut Vec<SortableLaunchOption>, text: &str) {
    let Some(context_lock) = get_context() else {
        return;
    };
    let Ok(mut context) = context_lock.write() else {
        return;
    };
    let eval = rink_core::eval(&mut context, text);

    if let Ok(eval) = eval {
        trace!("Eval: {eval:?}");
        for (title, desc) in parse_result(eval) {
            trace!("Added calc option: {title}, {desc:?}");
            matches.push(SortableLaunchOption {
                icon: Some(Box::from(Path::new("accessories-calculator"))),
                names: Box::from(vec![title.clone().into_boxed_str()]),
                details: desc.clone().into_boxed_str(),
                details_long: Some(Box::from("Copy to clipboard")),
                bonus_score: 5,
                enabled: true,
                takes_args: false,
                iden: Identifier::data(PluginName::Calc, title.into_boxed_str()),
                subactions: vec![],
            });
        }
    } else {
        trace!("No option added: expression error: {eval:?}");
    }
}

pub fn copy_result(data: Option<&str>) -> bool {
    use relm4::adw::gtk::prelude::DisplayExt;
    if let Some(data) = data
        && let Some(clipboard) =
            relm4::adw::gtk::gdk::Display::default().map(|display| display.clipboard())
    {
        debug!("Copying result to clipboard: {}", data);
        clipboard.set_text(data.as_ref());
    }
    false
}

#[allow(clippy::map_unwrap_or)]
#[tracing::instrument]
fn parse_result(result: QueryReply) -> Vec<(String, String)> {
    match result {
        QueryReply::Number(n) => {
            vec![tuple_from_np(&n)]
        }
        QueryReply::Date(d) => vec![(
            // TODO(db48x): we really should localize dates and times
            d.rfc3339,
            join(&[d.human], ""),
        )],
        QueryReply::Substance(s) => {
            s.properties
                .iter()
                .map(|p| (str_from_np(&p.value), p.name.clone()))
                .collect()
        }
        QueryReply::Duration(d) => {
            let parts = [
                d.years, d.months, d.weeks, d.days, d.hours, d.minutes, d.seconds,
            ]
            .iter()
            .cloned()
            .filter(|n| n.exact_value.as_deref() != Some("0"))
            .map(|n| n.raw_value.map(|n| str_from_np(&n.to_parts_simple())))
            .collect::<Vec<_>>();
            vec![(join(parts.as_slice(), ", "), join(&[d.raw.quantity], ""))]
        }
        QueryReply::Def(def) => vec![(
            def.to_string(),
            join(&[def.value.and_then(|v| v.quantity)], ""),
        )],
        QueryReply::Conversion(c) => vec![tuple_from_np(&c.value)],
        QueryReply::Factorize(f) => f
            .factorizations
            .iter()
            .map(|f| {
                (
                    f.units
                        .iter()
                        .map(|(u, &p)| pow(&u.clone(), p as i64))
                        .intersperse(String::from("⋅"))
                        .collect(),
                    String::from(""),
                )
            })
            .collect(),
        QueryReply::UnitsFor(f) => f
            .units
            .iter()
            .map(|u| {
                (
                    u.units.join(", "),
                    u.category.clone().unwrap_or_else(|| String::from("Other")),
                )
            })
            .collect(),
        QueryReply::UnitList(l) => {
            vec![(
                l.list
                    .iter()
                    .map(str_from_np)
                    .intersperse(String::from(", "))
                    .collect(),
                l.rest.quantity.unwrap_or_else(|| String::from("other")),
            )]
        }
        QueryReply::Search(s) => s.results.iter().map(tuple_from_np).collect(),
    }
}

fn join(parts: &[Option<String>], s: &str) -> String {
    parts
        .iter()
        .flatten()
        .filter(|s| !s.is_empty())
        .map(AsRef::as_ref)
        .collect::<Vec<_>>()
        .join(s)
}

fn str_from_np(n: &NumberParts) -> String {
    let n = n.clone();
    let frac_unit: Option<String> = match (&n.factor, &n.divfactor) {
        (None, None) => None,
        (Some(n), None) => Some(format!("× {n}")),
        (None, Some(d)) => Some(format!("× 1⁄{d}")),
        (Some(n), Some(d)) => Some(format!("× {n}⁄{d}")),
    };
    fn mkpow(x: (&BaseUnit, &i64)) -> String {
        pow(x.0.id.as_ref(), *x.1)
    }
    let pos_units = n.raw_unit.as_ref().map(|d| {
        d.iter()
            .filter(|(_, p)| **p > 0)
            .map(mkpow)
            .intersperse(String::from(" "))
            .collect()
    });
    let mut neg_units = n
        .raw_unit
        .as_ref()
        .map(|d| d.iter().filter(|(_, p)| **p < 0).collect::<Vec<_>>());
    let div = if let Some(ref u) = neg_units
        && u.len() >= 1
    {
        Some(String::from("/"))
    } else {
        None
    };
    let neg_units = if let Some(ref mut u) = neg_units
        && u.len() >= 1
    {
        Some(
            u.drain(0..)
                .map(mkpow)
                .intersperse(String::from(" "))
                .collect(),
        )
    } else {
        None
    };
    let dimensions = if n.raw_unit.is_some() {
        None
    } else {
        n.dimensions
    };
    let parts = &[
        n.approx_value.or(n.exact_value),
        frac_unit,
        dimensions,
        pos_units,
        div,
        neg_units,
    ];
    join(parts, " ")
}

fn tuple_from_np(n: &NumberParts) -> (String, String) {
    (str_from_np(&n), join(&[n.quantity.clone()], " "))
}

fn pow(n: &str, p: i64) -> String {
    if p == 1 || p == -1 {
        n.to_string()
    } else {
        let power = p.to_string();
        let digits = power.len();
        let mut n = n.to_string();
        n.reserve(4 * (digits + 1));
        power.chars().map(superscript_from_digit).for_each(|c| {
            c.inspect(|c| n.push(*c));
        });
        n
    }
}

fn superscript_from_digit(d: char) -> Option<char> {
    // From the Unicode "Superscripts and Subscripts" block, U+2070 to U+209F
    match d {
        '0' => Some('⁰'),
        '1' => Some('¹'),
        '2' => Some('²'),
        '3' => Some('³'),
        '4' => Some('⁴'),
        '5' => Some('⁵'),
        '6' => Some('⁶'),
        '7' => Some('⁷'),
        '8' => Some('⁸'),
        '9' => Some('⁹'),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use anyhow::{Context, Result};

    use crate::plugins::calc::{get_context, parse_result};

    /// workaround for the fact that rink’s QueryError doesn’t impl Error
    /// See <https://github.com/tiffany352/rink-rs/issues/238>.
    #[derive(Debug)]
    struct QueryError(rink_core::output::QueryError);

    impl std::fmt::Display for QueryError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl std::error::Error for QueryError {}

    impl From<rink_core::output::QueryError> for QueryError {
        fn from(value: rink_core::output::QueryError) -> Self {
            QueryError(value)
        }
    }

    fn eval(line: &str) -> Result<Vec<(String, String)>> {
        let mut context = get_context()
            .with_context(|| "unable to get rink context")?
            .write()
            .expect("lock is not poisoned");
        Ok(parse_result(
            rink_core::eval(&mut context, line).map_err(QueryError::from)?,
        ))
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_parse_result_simple() -> Result<()> {
        try {
            let result = eval("42")?;
            assert_eq!(result.len(), 1);
            assert_eq!(result[0].0, "42");
            assert_eq!(result[0].1, "dimensionless");
        }
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_parse_result_fraction() -> Result<()> {
        try {
            let result = eval("1/2")?;
            assert_eq!(result.len(), 1);
            assert_eq!(result[0].0, "0.5");
            assert_eq!(result[0].1, "dimensionless");
        }
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_parse_result_approx_with_dimensions() -> Result<()> {
        try {
            let result = eval("12|123 kg")?;
            assert_eq!(result.len(), 1);
            assert_eq!(result[0].0, "97.[56097]... gram");
            assert_eq!(result[0].1, "mass");
        }
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_parse_result_interesting_dimensions() -> Result<()> {
        try {
            let result = eval("1m * 1 m/s / 1s / 1s")?;
            assert_eq!(result.len(), 1);
            assert_eq!(result[0].0, "1 meter² / second³");
            assert_eq!(result[0].1, "absorbed_dose_rate");
        }
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_parse_result_date() -> Result<()> {
        try {
            let result = eval("#September 2, 1945#")?;
            assert_eq!(result.len(), 1);
            assert_eq!(result[0].0, "1945-09-02 00:00:00");
            // TODO(db48x): this depends on the current date and so should not be tested
            //assert_eq!(result[0].1, "80 years ago");
        }
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_parse_result_duration() -> Result<()> {
        try {
            let result = eval("1year+1day+1s")?;
            assert_eq!(result.len(), 1);
            assert_eq!(result[0].0, "1 year, 1 day, 1 second");
            assert_eq!(result[0].1, "time");
        }
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_parse_result_conversion() -> Result<()> {
        try {
            let result = eval("1 kg → gram")?;
            assert_eq!(result.len(), 1);
            assert_eq!(result[0].0, "1000 gram");
            assert_eq!(result[0].1, "mass");
        }
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_parse_result_conversion_with_fractional_dimensions() -> Result<()> {
        try {
            let result = eval("1m → 21|32ft")?;
            assert_eq!(result.len(), 1);
            assert_eq!(result[0].0, "4.999375 × 21⁄32 foot");
            assert_eq!(result[0].1, "length");
        }
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_parse_result_unit_list() -> Result<()> {
        try {
            let result = eval(
                "1month → siderealmonth;fortnight;watch;decimalminute;blink;millisecond;microsecond;shake",
            )?;
            assert_eq!(result.len(), 1);
            assert_eq!(
                result[0].0,
                "1 siderealmonth, 0 fortnight, 18 watch, 115 decimalminute, 18 blink, 768 millisecond, 823 microsecond, 20 shake"
            );
            assert_eq!(result[0].1, "time");
        }
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_parse_result_definition() -> Result<()> {
        try {
            let result = eval("erg")?;
            assert_eq!(result.len(), 1);
            assert_eq!(
                result[0].0,
                "Definition: erg = cm dyne = 100 nanojoule (energy; kg m^2 / s^2)"
            );
            assert_eq!(result[0].1, "energy");
        }
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_parse_result_substance() -> Result<()> {
        try {
            let result = eval("hydrogen")?;
            assert_eq!(result.len(), 3);
            let hash = result
                .iter()
                .cloned()
                .map(|p| (p.1, p.0))
                .collect::<HashMap<_, _>>();
            assert_eq!(hash["atomic_number"], "1");
            assert_eq!(hash["molar_mass"], "1.00794 gram / mole");
            assert_eq!(hash["specific_heat"], "14300 meter^2 / kelvin second^2");
        }
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_parse_result_factorize() -> Result<()> {
        try {
            let results = eval("factorize velocity")?;
            assert_eq!(results.len(), 5);
            assert!(results.iter().any(|(f, _)| f == "acceleration⋅time"));
            assert!(results.iter().any(|(f, _)| f == "jerk⋅time²"));
        }
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_parse_result_search() -> Result<()> {
        try {
            let results = eval("search milk")?;
            assert_eq!(results.len(), 5);
            assert!(results.contains(&(String::from("milk"), String::from("substance"))));
            assert!(results.contains(&(String::from("mil"), String::from("length"))));
            assert!(results.contains(&(String::from("mile"), String::from("length"))));
            assert!(results.contains(&(String::from("mill"), String::from("dimensionless"))));
            assert!(results.contains(&(String::from("mi"), String::from("length"))));
        }
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_parse_result_units_for() -> Result<()> {
        try {
            let results = eval("units for velocity")?;
            assert_eq!(results.len(), 9);
            assert!(results.contains(&(
                String::from("fpm, fps, ipy, kmh, kph, mph"),
                String::from("Abbreviations")
            )));
            assert!(results.contains(&(
                String::from("brknot"),
                String::from("British Length Measures")
            )));
            assert!(results.contains(&(String::from("kine"), String::from("CGS Units"))));
            assert!(results.contains(&(String::from("㎧"), String::from("Unicode aliases"))));
            assert!(
                results.contains(&(String::from("c, mach"), String::from("Physical Constants")))
            );
        }
    }
}
