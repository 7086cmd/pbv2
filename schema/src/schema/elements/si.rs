//! # SI Units / siunitx support
//!
//! This module mirrors the [`siunitx`](https://ctan.org/pkg/siunitx) LaTeX package.
//! It provides:
//!
//! * [`SiPrefix`] – all SI metric prefixes (quecto … quetta).
//! * [`SIUnit`] – every named unit recognised by siunitx, plus a small set of
//!   commonly used non-SI units.
//! * [`UnitFactor`] – a single (optionally prefixed) unit raised to an integer
//!   power.
//! * [`CompoundUnit`] – an ordered product of [`UnitFactor`]s, sufficient to
//!   express any SI unit expression including `\per` denominators.
//! * [`SiUnitX`] – the full set of siunitx *quantity/number/unit* commands:
//!   `\num`, `\numlist`, `\numrange`, `\numproduct`, `\ang`, `\unit`, `\qty`,
//!   `\qtyrange`, `\qtylist`, `\qtyproduct`, `\complexnum`, `\complexqty`.
//!
//! Every type implements `Renderer<Latex, Universal>`, `Renderer<Html, Universal>`,
//! and `Renderer<Markdown, Universal>` so they can be embedded in any [`Element`].

use crate::schema::renderer::{Html, Latex, Markdown, Renderer, Universal};

// ── SiPrefix ──────────────────────────────────────────────────────────────────

/// All SI metric prefixes, from quecto (10⁻³⁰) to quetta (10³⁰).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SiPrefix {
    Quecto, // 10^-30
    Ronto,  // 10^-27
    Yocto,  // 10^-24
    Zepto,  // 10^-21
    Atto,   // 10^-18
    Femto,  // 10^-15
    Pico,   // 10^-12
    Nano,   // 10^-9
    Micro,  // 10^-6
    Milli,  // 10^-3
    Centi,  // 10^-2
    Deci,   // 10^-1
    Deca,   // 10^1
    Hecto,  // 10^2
    Kilo,   // 10^3
    Mega,   // 10^6
    Giga,   // 10^9
    Tera,   // 10^12
    Peta,   // 10^15
    Exa,    // 10^18
    Zetta,  // 10^21
    Yotta,  // 10^24
    Ronna,  // 10^27
    Quetta, // 10^30
}

impl SiPrefix {
    /// The siunitx LaTeX command for this prefix (e.g. `\kilo`).
    pub fn to_latex(self) -> &'static str {
        match self {
            SiPrefix::Quecto => "\\quecto",
            SiPrefix::Ronto  => "\\ronto",
            SiPrefix::Yocto  => "\\yocto",
            SiPrefix::Zepto  => "\\zepto",
            SiPrefix::Atto   => "\\atto",
            SiPrefix::Femto  => "\\femto",
            SiPrefix::Pico   => "\\pico",
            SiPrefix::Nano   => "\\nano",
            SiPrefix::Micro  => "\\micro",
            SiPrefix::Milli  => "\\milli",
            SiPrefix::Centi  => "\\centi",
            SiPrefix::Deci   => "\\deci",
            SiPrefix::Deca   => "\\deca",
            SiPrefix::Hecto  => "\\hecto",
            SiPrefix::Kilo   => "\\kilo",
            SiPrefix::Mega   => "\\mega",
            SiPrefix::Giga   => "\\giga",
            SiPrefix::Tera   => "\\tera",
            SiPrefix::Peta   => "\\peta",
            SiPrefix::Exa    => "\\exa",
            SiPrefix::Zetta  => "\\zetta",
            SiPrefix::Yotta  => "\\yotta",
            SiPrefix::Ronna  => "\\ronna",
            SiPrefix::Quetta => "\\quetta",
        }
    }

    /// The standard SI symbol for this prefix (e.g. `k` for kilo, `μ` for micro).
    pub fn symbol(self) -> &'static str {
        match self {
            SiPrefix::Quecto => "q",
            SiPrefix::Ronto  => "r",
            SiPrefix::Yocto  => "y",
            SiPrefix::Zepto  => "z",
            SiPrefix::Atto   => "a",
            SiPrefix::Femto  => "f",
            SiPrefix::Pico   => "p",
            SiPrefix::Nano   => "n",
            SiPrefix::Micro  => "μ",
            SiPrefix::Milli  => "m",
            SiPrefix::Centi  => "c",
            SiPrefix::Deci   => "d",
            SiPrefix::Deca   => "da",
            SiPrefix::Hecto  => "h",
            SiPrefix::Kilo   => "k",
            SiPrefix::Mega   => "M",
            SiPrefix::Giga   => "G",
            SiPrefix::Tera   => "T",
            SiPrefix::Peta   => "P",
            SiPrefix::Exa    => "E",
            SiPrefix::Zetta  => "Z",
            SiPrefix::Yotta  => "Y",
            SiPrefix::Ronna  => "R",
            SiPrefix::Quetta => "Q",
        }
    }

    /// The base-10 exponent for this prefix.
    pub fn exponent(self) -> i32 {
        match self {
            SiPrefix::Quecto => -30,
            SiPrefix::Ronto  => -27,
            SiPrefix::Yocto  => -24,
            SiPrefix::Zepto  => -21,
            SiPrefix::Atto   => -18,
            SiPrefix::Femto  => -15,
            SiPrefix::Pico   => -12,
            SiPrefix::Nano   => -9,
            SiPrefix::Micro  => -6,
            SiPrefix::Milli  => -3,
            SiPrefix::Centi  => -2,
            SiPrefix::Deci   => -1,
            SiPrefix::Deca   => 1,
            SiPrefix::Hecto  => 2,
            SiPrefix::Kilo   => 3,
            SiPrefix::Mega   => 6,
            SiPrefix::Giga   => 9,
            SiPrefix::Tera   => 12,
            SiPrefix::Peta   => 15,
            SiPrefix::Exa    => 18,
            SiPrefix::Zetta  => 21,
            SiPrefix::Yotta  => 24,
            SiPrefix::Ronna  => 27,
            SiPrefix::Quetta => 30,
        }
    }
}

// ── SIUnit ────────────────────────────────────────────────────────────────────

/// Named units recognised by siunitx (base, derived, and commonly used non-SI).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SIUnit {
    // ── SI base units (7) ──────────────────────────────────────────────────
    Meter,
    Kilogram,
    Second,
    Ampere,
    Kelvin,
    Mole,
    Candela,

    // ── SI derived units with special names (22) ──────────────────────────
    Radian,
    Steradian,
    Hertz,
    Newton,
    Pascal,
    Joule,
    Watt,
    Coulomb,
    Volt,
    Farad,
    Ohm,
    Siemens,
    Weber,
    Tesla,
    Henry,
    Lumen,
    Lux,
    Becquerel,
    Gray,
    Sievert,
    Katal,

    // ── Non-SI units accepted for use with SI ─────────────────────────────
    Minute,           // min  (time)
    Hour,             // h
    Day,              // d
    AstronomicalUnit, // au
    Degree,           // ° (plane angle)
    Arcminute,        // ′
    Arcsecond,        // ″
    Hectare,          // ha
    Litre,            // L  (also spelled Liter)
    Tonne,            // t  (metric ton)
    Dalton,           // Da
    Electronvolt,     // eV

    // ── Other commonly used units ─────────────────────────────────────────
    Bar,             // bar
    Atmosphere,      // atm
    Celsius,         // °C  (\degreeCelsius in siunitx)
    AtomicMassUnit,  // u   (\atomicmassunit)
    Neper,           // Np
    Bel,             // B
    Decibel,         // dB
    Percent,         // %
}

impl SIUnit {
    /// The siunitx LaTeX command for this unit (e.g. `\meter`, `\kilogram`).
    pub fn to_latex(&self) -> &'static str {
        match self {
            SIUnit::Meter            => "\\meter",
            SIUnit::Kilogram         => "\\kilogram",
            SIUnit::Second           => "\\second",
            SIUnit::Ampere           => "\\ampere",
            SIUnit::Kelvin           => "\\kelvin",
            SIUnit::Mole             => "\\mole",
            SIUnit::Candela          => "\\candela",
            SIUnit::Radian           => "\\radian",
            SIUnit::Steradian        => "\\steradian",
            SIUnit::Hertz            => "\\hertz",
            SIUnit::Newton           => "\\newton",
            SIUnit::Pascal           => "\\pascal",
            SIUnit::Joule            => "\\joule",
            SIUnit::Watt             => "\\watt",
            SIUnit::Coulomb          => "\\coulomb",
            SIUnit::Volt             => "\\volt",
            SIUnit::Farad            => "\\farad",
            SIUnit::Ohm              => "\\ohm",
            SIUnit::Siemens          => "\\siemens",
            SIUnit::Weber            => "\\weber",
            SIUnit::Tesla            => "\\tesla",
            SIUnit::Henry            => "\\henry",
            SIUnit::Lumen            => "\\lumen",
            SIUnit::Lux              => "\\lux",
            SIUnit::Becquerel        => "\\becquerel",
            SIUnit::Gray             => "\\gray",
            SIUnit::Sievert          => "\\sievert",
            SIUnit::Katal            => "\\katal",
            SIUnit::Minute           => "\\minute",
            SIUnit::Hour             => "\\hour",
            SIUnit::Day              => "\\day",
            SIUnit::AstronomicalUnit => "\\astronomicalunit",
            SIUnit::Degree           => "\\degree",
            SIUnit::Arcminute        => "\\arcminute",
            SIUnit::Arcsecond        => "\\arcsecond",
            SIUnit::Hectare          => "\\hectare",
            SIUnit::Litre            => "\\litre",
            SIUnit::Tonne            => "\\tonne",
            SIUnit::Dalton           => "\\dalton",
            SIUnit::Electronvolt     => "\\electronvolt",
            SIUnit::Bar              => "\\bar",
            SIUnit::Atmosphere       => "\\atm",
            SIUnit::Celsius          => "\\degreeCelsius",
            SIUnit::AtomicMassUnit   => "\\atomicmassunit",
            SIUnit::Neper            => "\\neper",
            SIUnit::Bel              => "\\bel",
            SIUnit::Decibel          => "\\decibel",
            SIUnit::Percent          => "\\percent",
        }
    }

    /// The standard symbol for this unit as a Unicode string (e.g. `"m"`, `"Ω"`).
    pub fn symbol(&self) -> &'static str {
        match self {
            SIUnit::Meter            => "m",
            SIUnit::Kilogram         => "kg",
            SIUnit::Second           => "s",
            SIUnit::Ampere           => "A",
            SIUnit::Kelvin           => "K",
            SIUnit::Mole             => "mol",
            SIUnit::Candela          => "cd",
            SIUnit::Radian           => "rad",
            SIUnit::Steradian        => "sr",
            SIUnit::Hertz            => "Hz",
            SIUnit::Newton           => "N",
            SIUnit::Pascal           => "Pa",
            SIUnit::Joule            => "J",
            SIUnit::Watt             => "W",
            SIUnit::Coulomb          => "C",
            SIUnit::Volt             => "V",
            SIUnit::Farad            => "F",
            SIUnit::Ohm              => "Ω",
            SIUnit::Siemens          => "S",
            SIUnit::Weber            => "Wb",
            SIUnit::Tesla            => "T",
            SIUnit::Henry            => "H",
            SIUnit::Lumen            => "lm",
            SIUnit::Lux              => "lx",
            SIUnit::Becquerel        => "Bq",
            SIUnit::Gray             => "Gy",
            SIUnit::Sievert          => "Sv",
            SIUnit::Katal            => "kat",
            SIUnit::Minute           => "min",
            SIUnit::Hour             => "h",
            SIUnit::Day              => "d",
            SIUnit::AstronomicalUnit => "au",
            SIUnit::Degree           => "°",
            SIUnit::Arcminute        => "′",
            SIUnit::Arcsecond        => "″",
            SIUnit::Hectare          => "ha",
            SIUnit::Litre            => "L",
            SIUnit::Tonne            => "t",
            SIUnit::Dalton           => "Da",
            SIUnit::Electronvolt     => "eV",
            SIUnit::Bar              => "bar",
            SIUnit::Atmosphere       => "atm",
            SIUnit::Celsius          => "°C",
            SIUnit::AtomicMassUnit   => "u",
            SIUnit::Neper            => "Np",
            SIUnit::Bel              => "B",
            SIUnit::Decibel          => "dB",
            SIUnit::Percent          => "%",
        }
    }
}

// ── UnitFactor & CompoundUnit ─────────────────────────────────────────────────

/// One unit in a compound expression, e.g. `\kilo\meter\squared` is
/// `UnitFactor { prefix: Some(Kilo), unit: Meter, power: 2 }`.
///
/// A negative `power` corresponds to a denominator factor (`\per`).
#[derive(Debug, Clone, PartialEq)]
pub struct UnitFactor {
    pub prefix: Option<SiPrefix>,
    pub unit: SIUnit,
    /// Integer exponent. Positive → numerator; negative → denominator.
    pub power: i32,
}

impl UnitFactor {
    pub fn new(unit: SIUnit) -> Self {
        Self { prefix: None, unit, power: 1 }
    }
    pub fn with_prefix(mut self, prefix: SiPrefix) -> Self {
        self.prefix = Some(prefix);
        self
    }
    pub fn with_power(mut self, power: i32) -> Self {
        self.power = power;
        self
    }

    // ── Rendering helpers ─────────────────────────────────────────────────

    /// Produces the siunitx fragment for this factor inside `\unit{…}` /
    /// `\qty{…}{…}`, using `\per`, `\squared`, `\cubed`, and `\tothe{n}`.
    pub fn to_latex_fragment(&self) -> String {
        let mut s = String::new();
        let abs_power = self.power.unsigned_abs();
        if self.power < 0 {
            s.push_str("\\per");
        }
        if let Some(p) = self.prefix {
            s.push_str(p.to_latex());
        }
        s.push_str(self.unit.to_latex());
        match abs_power {
            1 => {}
            2 => s.push_str("\\squared"),
            3 => s.push_str("\\cubed"),
            n => s.push_str(&format!("\\tothe{{{}}}", n)),
        }
        s
    }

    /// Symbol string for plain-text targets (HTML / Markdown), e.g. `km²`.
    fn symbol_str(&self) -> String {
        let prefix_sym = self.prefix.map(|p| p.symbol()).unwrap_or("");
        let unit_sym = self.unit.symbol();
        let power_str = match self.power {
            1  => String::new(),
            2  => "²".to_owned(),
            3  => "³".to_owned(),
            -1 => "⁻¹".to_owned(),
            -2 => "⁻²".to_owned(),
            -3 => "⁻³".to_owned(),
            n  => format!("^{}", n),
        };
        format!("{}{}{}", prefix_sym, unit_sym, power_str)
    }

    /// HTML fragment for this unit factor.
    fn to_html(&self) -> String {
        let prefix_sym = self.prefix.map(|p| p.symbol()).unwrap_or("");
        let unit_sym = self.unit.symbol();
        let abs_power = self.power.abs();
        let power_html = if abs_power == 1 {
            String::new()
        } else {
            format!("<sup>{}</sup>", abs_power)
        };
        let negative = if self.power < 0 { "<sup>-</sup>" } else { "" };
        format!("{}{}{}{}", negative, prefix_sym, unit_sym, power_html)
    }
}

/// An ordered product of [`UnitFactor`]s. This is the unit type used in all
/// siunitx quantity commands.
///
/// # Examples
/// ```no_run
/// // m/s  →  CompoundUnit { factors: [UnitFactor(Meter,1), UnitFactor(Second,-1)] }
/// // kg⋅m/s²  →  factors: [Kilogram/1, Meter/1, Second/-2]
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct CompoundUnit {
    pub factors: Vec<UnitFactor>,
}

impl CompoundUnit {
    pub fn new(factors: Vec<UnitFactor>) -> Self {
        Self { factors }
    }

    /// Single-unit shorthand.
    pub fn single(unit: SIUnit) -> Self {
        Self { factors: vec![UnitFactor::new(unit)] }
    }

    /// Full siunitx LaTeX string like `\kilo\meter\per\second\squared`.
    pub fn to_latex_str(&self) -> String {
        self.factors.iter().map(UnitFactor::to_latex_fragment).collect()
    }

    /// Plain symbol string like `km⋅s⁻²`, separating positive-power factors
    /// with `⋅` and grouping negative-power ones after `/`.
    fn to_symbol_str(&self) -> String {
        let num: Vec<String> = self.factors.iter()
            .filter(|f| f.power >= 0)
            .map(UnitFactor::symbol_str)
            .collect();
        let den: Vec<String> = self.factors.iter()
            .filter(|f| f.power < 0)
            .map(|f| {
                let pos = UnitFactor { power: -f.power, ..f.clone() };
                pos.symbol_str()
            })
            .collect();
        match (num.is_empty(), den.is_empty()) {
            (false, true)  => num.join("⋅"),
            (true,  false) => format!("1/{}", den.join("⋅")),
            (false, false) => format!("{}/{}", num.join("⋅"), den.join("⋅")),
            (true,  true)  => String::new(),
        }
    }

    /// HTML string, numerator joined with `&middot;` and denominator after `/`.
    fn to_html_str(&self) -> String {
        let num: Vec<String> = self.factors.iter()
            .filter(|f| f.power >= 0)
            .map(UnitFactor::to_html)
            .collect();
        let den: Vec<String> = self.factors.iter()
            .filter(|f| f.power < 0)
            .map(|f| {
                let pos = UnitFactor { power: -f.power, ..f.clone() };
                pos.to_html()
            })
            .collect();
        match (num.is_empty(), den.is_empty()) {
            (false, true)  => num.join("&middot;"),
            (true,  false) => format!("1/{}", den.join("&middot;")),
            (false, false) => format!("{}/{}", num.join("&middot;"), den.join("&middot;")),
            (true,  true)  => String::new(),
        }
    }
}

// ── SiUnitX ───────────────────────────────────────────────────────────────────

/// The complete set of siunitx number/unit/quantity commands.
#[derive(Debug, Clone, PartialEq)]
pub enum SiUnitX {
    // ── Number commands ───────────────────────────────────────────────────

    /// `\num{value}` – a dimensionless number with siunitx formatting.
    Number { value: f64 },

    /// `\numlist{v1; v2; v3; ...}` – a list of numbers.
    NumberList { values: Vec<f64> },

    /// `\numrange{min}{max}` – a range of numbers.
    NumberRange { min: f64, max: f64 },

    /// `\numproduct{v1 x v2 x ...}` – a product of numbers.
    NumberProduct { factors: Vec<f64> },

    // ── Angle commands ────────────────────────────────────────────────────

    /// `\ang{value}` – an angle in decimal degrees.
    Angle { value: f64 },

    /// `\ang{deg;min;sec}` – an angle in degrees/minutes/seconds (DMS).
    AngleDMS {
        degrees: f64,
        minutes: Option<f64>,
        seconds: Option<f64>,
    },

    // ── Unit commands ─────────────────────────────────────────────────────

    /// `\unit{unit}` (also `\si{unit}`) – a unit without a numerical value.
    Unit { unit: CompoundUnit },

    // ── Quantity commands ─────────────────────────────────────────────────

    /// `\qty{value}{unit}` (also `\SI{value}{unit}`) – a quantity.
    Quantity { value: f64, unit: CompoundUnit },

    /// `\qtyrange{min}{max}{unit}` – a range of quantities.
    QuantityRange { min: f64, max: f64, unit: CompoundUnit },

    /// `\qtylist{v1; v2; v3; ...}{unit}` – a list of quantities.
    QuantityList { values: Vec<f64>, unit: CompoundUnit },

    /// `\qtyproduct{v1 x v2 x ...}{unit}` – a product of quantities.
    QuantityProduct { factors: Vec<f64>, unit: CompoundUnit },

    // ── Complex number / quantity commands ────────────────────────────────

    /// `\complexnum{real+imagi}` – a complex number.
    ComplexNum { real: f64, imag: f64 },

    /// `\complexqty{real+imagi}{unit}` – a complex quantity.
    ComplexQty { real: f64, imag: f64, unit: CompoundUnit },
}

// ── Internal formatting helpers ───────────────────────────────────────────────

/// Format an `f64` to a clean string: drops trailing `.0` from whole numbers.
fn fmt_num(v: f64) -> String {
    if v.fract() == 0.0 && v.abs() < 1e15 {
        format!("{:.0}", v)
    } else {
        format!("{}", v)
    }
}

/// Build a plain-text number list separated by `sep`.
fn num_list_text(values: &[f64], sep: &str) -> String {
    values.iter().map(|v| fmt_num(*v)).collect::<Vec<_>>().join(sep)
}

// ── Renderer<Latex, Universal> for SiUnitX ───────────────────────────────────

impl Renderer<Latex, Universal> for SiUnitX {
    fn render(&self) -> anyhow::Result<String> {
        let s = match self {
            // ── numbers ───────────────────────────────────────────────────
            SiUnitX::Number { value } =>
                format!("\\num{{{}}}", fmt_num(*value)),

            SiUnitX::NumberList { values } => {
                let list = values.iter().map(|v| fmt_num(*v)).collect::<Vec<_>>().join("; ");
                format!("\\numlist{{{}}}", list)
            }

            SiUnitX::NumberRange { min, max } =>
                format!("\\numrange{{{}}}{{{}}}", fmt_num(*min), fmt_num(*max)),

            SiUnitX::NumberProduct { factors } => {
                let prod = factors.iter().map(|v| fmt_num(*v)).collect::<Vec<_>>().join(" x ");
                format!("\\numproduct{{{}}}", prod)
            }

            // ── angles ───────────────────────────────────────────────────
            SiUnitX::Angle { value } =>
                format!("\\ang{{{}}}", fmt_num(*value)),

            SiUnitX::AngleDMS { degrees, minutes, seconds } => {
                let d = fmt_num(*degrees);
                let m = minutes.map(|v| fmt_num(v)).unwrap_or_default();
                let s = seconds.map(|v| fmt_num(v)).unwrap_or_default();
                format!("\\ang{{{};{};{}}}", d, m, s)
            }

            // ── units ─────────────────────────────────────────────────────
            SiUnitX::Unit { unit } =>
                format!("\\unit{{{}}}", unit.to_latex_str()),

            // ── quantities ────────────────────────────────────────────────
            SiUnitX::Quantity { value, unit } =>
                format!("\\qty{{{}}}{{{}}}", fmt_num(*value), unit.to_latex_str()),

            SiUnitX::QuantityRange { min, max, unit } =>
                format!("\\qtyrange{{{}}}{{{}}}{{{}}}", fmt_num(*min), fmt_num(*max), unit.to_latex_str()),

            SiUnitX::QuantityList { values, unit } => {
                let list = values.iter().map(|v| fmt_num(*v)).collect::<Vec<_>>().join("; ");
                format!("\\qtylist{{{}}}{{{}}}", list, unit.to_latex_str())
            }

            SiUnitX::QuantityProduct { factors, unit } => {
                let prod = factors.iter().map(|v| fmt_num(*v)).collect::<Vec<_>>().join(" x ");
                format!("\\qtyproduct{{{}}}{{{}}}", prod, unit.to_latex_str())
            }

            // ── complex ───────────────────────────────────────────────────
            SiUnitX::ComplexNum { real, imag } => {
                let sign = if *imag >= 0.0 { "+" } else { "" };
                format!("\\complexnum{{{}{}{}{}}}", fmt_num(*real), sign, fmt_num(*imag), "i")
            }

            SiUnitX::ComplexQty { real, imag, unit } => {
                let sign = if *imag >= 0.0 { "+" } else { "" };
                let num_str = format!("{}{}{}{}", fmt_num(*real), sign, fmt_num(*imag), "i");
                format!("\\complexqty{{{}}}{{{}}}", num_str, unit.to_latex_str())
            }
        };
        Ok(s)
    }
}

// ── Renderer<Html, Universal> for SiUnitX ────────────────────────────────────

impl Renderer<Html, Universal> for SiUnitX {
    fn render(&self) -> anyhow::Result<String> {
        let s = match self {
            SiUnitX::Number { value } =>
                format!("<span class=\"si-num\">{}</span>", fmt_num(*value)),

            SiUnitX::NumberList { values } => {
                let parts: Vec<String> = values.iter().map(|v| fmt_num(*v)).collect();
                format!("<span class=\"si-numlist\">{}</span>", parts.join(", "))
            }

            SiUnitX::NumberRange { min, max } =>
                format!("<span class=\"si-numrange\">{}&ndash;{}</span>",
                    fmt_num(*min), fmt_num(*max)),

            SiUnitX::NumberProduct { factors } => {
                let parts: Vec<String> = factors.iter().map(|v| fmt_num(*v)).collect();
                format!("<span class=\"si-numproduct\">{}</span>", parts.join(" &times; "))
            }

            SiUnitX::Angle { value } =>
                format!("<span class=\"si-ang\">{}&deg;</span>", fmt_num(*value)),

            SiUnitX::AngleDMS { degrees, minutes, seconds } => {
                let mut parts = format!("{}&deg;", fmt_num(*degrees));
                if let Some(m) = minutes { parts.push_str(&format!("{}&#8242;", fmt_num(*m))); }
                if let Some(s) = seconds { parts.push_str(&format!("{}&#8243;", fmt_num(*s))); }
                format!("<span class=\"si-ang\">{}</span>", parts)
            }

            SiUnitX::Unit { unit } =>
                format!("<span class=\"si-unit\">{}</span>", unit.to_html_str()),

            SiUnitX::Quantity { value, unit } =>
                format!("<span class=\"si-qty\">{}&thinsp;{}</span>",
                    fmt_num(*value), unit.to_html_str()),

            SiUnitX::QuantityRange { min, max, unit } =>
                format!("<span class=\"si-qtyrange\">{}&ndash;{}&thinsp;{}</span>",
                    fmt_num(*min), fmt_num(*max), unit.to_html_str()),

            SiUnitX::QuantityList { values, unit } => {
                let parts: Vec<String> = values.iter().map(|v| fmt_num(*v)).collect();
                format!("<span class=\"si-qtylist\">{}&thinsp;{}</span>",
                    parts.join(", "), unit.to_html_str())
            }

            SiUnitX::QuantityProduct { factors, unit } => {
                let parts: Vec<String> = factors.iter().map(|v| fmt_num(*v)).collect();
                format!("<span class=\"si-qtyproduct\">{}&thinsp;{}</span>",
                    parts.join(" &times; "), unit.to_html_str())
            }

            SiUnitX::ComplexNum { real, imag } => {
                let sign = if *imag >= 0.0 { "+" } else { "" };
                format!("<span class=\"si-complexnum\">{}{}{}<i>i</i></span>",
                    fmt_num(*real), sign, fmt_num(*imag))
            }

            SiUnitX::ComplexQty { real, imag, unit } => {
                let sign = if *imag >= 0.0 { "+" } else { "" };
                format!("<span class=\"si-complexqty\">({}{}{}<i>i</i>)&thinsp;{}</span>",
                    fmt_num(*real), sign, fmt_num(*imag), unit.to_html_str())
            }
        };
        Ok(s)
    }
}

// ── Renderer<Markdown, Universal> for SiUnitX ────────────────────────────────

impl Renderer<Markdown, Universal> for SiUnitX {
    fn render(&self) -> anyhow::Result<String> {
        let s = match self {
            SiUnitX::Number { value } =>
                fmt_num(*value),

            SiUnitX::NumberList { values } =>
                num_list_text(values, ", "),

            SiUnitX::NumberRange { min, max } =>
                format!("{}–{}", fmt_num(*min), fmt_num(*max)),

            SiUnitX::NumberProduct { factors } =>
                factors.iter().map(|v| fmt_num(*v)).collect::<Vec<_>>().join(" × "),

            SiUnitX::Angle { value } =>
                format!("{}°", fmt_num(*value)),

            SiUnitX::AngleDMS { degrees, minutes, seconds } => {
                let mut s = format!("{}°", fmt_num(*degrees));
                if let Some(m) = minutes { s.push_str(&format!("{}′", fmt_num(*m))); }
                if let Some(sec) = seconds { s.push_str(&format!("{}″", fmt_num(*sec))); }
                s
            }

            SiUnitX::Unit { unit } =>
                unit.to_symbol_str(),

            // U+202F = NARROW NO-BREAK SPACE (standard between value and unit)
            SiUnitX::Quantity { value, unit } =>
                format!("{}\u{202F}{}", fmt_num(*value), unit.to_symbol_str()),

            SiUnitX::QuantityRange { min, max, unit } =>
                format!("{}–{}\u{202F}{}", fmt_num(*min), fmt_num(*max), unit.to_symbol_str()),

            SiUnitX::QuantityList { values, unit } =>
                format!("{}\u{202F}{}",
                    values.iter().map(|v| fmt_num(*v)).collect::<Vec<_>>().join(", "),
                    unit.to_symbol_str()),

            SiUnitX::QuantityProduct { factors, unit } =>
                format!("{}\u{202F}{}",
                    factors.iter().map(|v| fmt_num(*v)).collect::<Vec<_>>().join(" × "),
                    unit.to_symbol_str()),

            SiUnitX::ComplexNum { real, imag } => {
                let sign = if *imag >= 0.0 { "+" } else { "" };
                format!("{}{}{}i", fmt_num(*real), sign, fmt_num(*imag))
            }

            SiUnitX::ComplexQty { real, imag, unit } => {
                let sign = if *imag >= 0.0 { "+" } else { "" };
                format!("({}{}{}i)\u{202F}{}",
                    fmt_num(*real), sign, fmt_num(*imag), unit.to_symbol_str())
            }
        };
        Ok(s)
    }
}

// ── Renderer impls for CompoundUnit ──────────────────────────────────────────
//
// Useful when a `CompoundUnit` itself needs to be embedded in an element.

impl Renderer<Latex, Universal> for CompoundUnit {
    fn render(&self) -> anyhow::Result<String> {
        Ok(format!("\\unit{{{}}}", self.to_latex_str()))
    }
}

impl Renderer<Html, Universal> for CompoundUnit {
    fn render(&self) -> anyhow::Result<String> {
        Ok(format!("<span class=\"si-unit\">{}</span>", self.to_html_str()))
    }
}

impl Renderer<Markdown, Universal> for CompoundUnit {
    fn render(&self) -> anyhow::Result<String> {
        Ok(self.to_symbol_str())
    }
}

// ── From conversions ──────────────────────────────────────────────────────────

impl From<SIUnit> for CompoundUnit {
    fn from(unit: SIUnit) -> Self {
        CompoundUnit::single(unit)
    }
}

impl From<UnitFactor> for CompoundUnit {
    fn from(factor: UnitFactor) -> Self {
        CompoundUnit::new(vec![factor])
    }
}

// ── FromStr / from_latex parsing ──────────────────────────────────────────────
//
// Parses the full set of siunitx LaTeX commands back into `SiUnitX` values:
//   \num, \numlist, \numrange, \numproduct,
//   \ang,
//   \unit / \si,
//   \qty / \SI, \qtyrange, \qtylist, \qtyproduct,
//   \complexnum, \complexqty

impl SiUnitX {
    /// Returns `true` when `s` (trimmed) starts with a recognised siunitx
    /// command such as `\num`, `\qty`, `\ang`, etc.  Useful as a fast gate
    /// before attempting a full parse.
    pub fn is_si_command(s: &str) -> bool {
        let s = s.trim();
        for cmd in &[
            "\\num{", "\\numlist{", "\\numrange{", "\\numproduct{",
            "\\ang{",
            "\\unit{", "\\si{",
            "\\qty{", "\\SI{",
            "\\qtyrange{", "\\qtylist{", "\\qtyproduct{",
            "\\complexnum{", "\\complexqty{",
        ] {
            if s.starts_with(cmd) {
                return true;
            }
        }
        false
    }

    /// Parse a siunitx LaTeX command string into a `SiUnitX` value.
    ///
    /// Accepts the full set of siunitx quantity/number/unit macros.  All
    /// standard argument forms are supported including scientific-notation
    /// numbers (`1.5e3`) and compound unit expressions
    /// (`\kilo\meter\per\second\squared`).
    pub fn from_latex(s: &str) -> anyhow::Result<Self> {
        s.parse()
    }
}

impl std::str::FromStr for SiUnitX {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        let s = s.trim();

        // ── \numproduct (must check before \num) ──────────────────────────
        if let Some(rest) = s.strip_prefix("\\numproduct") {
            let (arg, _) = grab_brace_arg(rest)
                .ok_or_else(|| anyhow::anyhow!("bad \\numproduct arg"))?;
            let factors = parse_num_product(arg)?;
            return Ok(SiUnitX::NumberProduct { factors });
        }
        // ── \numlist ──────────────────────────────────────────────────────
        if let Some(rest) = s.strip_prefix("\\numlist") {
            let (arg, _) = grab_brace_arg(rest)
                .ok_or_else(|| anyhow::anyhow!("bad \\numlist arg"))?;
            let values = parse_num_semicolon_list(arg)?;
            return Ok(SiUnitX::NumberList { values });
        }
        // ── \numrange ─────────────────────────────────────────────────────
        if let Some(rest) = s.strip_prefix("\\numrange") {
            let (a1, rest) = grab_brace_arg(rest)
                .ok_or_else(|| anyhow::anyhow!("bad \\numrange arg1"))?;
            let (a2, _) = grab_brace_arg(rest.trim_start())
                .ok_or_else(|| anyhow::anyhow!("bad \\numrange arg2"))?;
            return Ok(SiUnitX::NumberRange {
                min: parse_si_num(a1)?,
                max: parse_si_num(a2)?,
            });
        }
        // ── \num ──────────────────────────────────────────────────────────
        if let Some(rest) = s.strip_prefix("\\num") {
            let (arg, _) = grab_brace_arg(rest)
                .ok_or_else(|| anyhow::anyhow!("bad \\num arg"))?;
            return Ok(SiUnitX::Number { value: parse_si_num(arg)? });
        }
        // ── \ang ──────────────────────────────────────────────────────────
        if let Some(rest) = s.strip_prefix("\\ang") {
            let (arg, _) = grab_brace_arg(rest)
                .ok_or_else(|| anyhow::anyhow!("bad \\ang arg"))?;
            // DMS if contains `;`, otherwise decimal
            if arg.contains(';') {
                let parts: Vec<&str> = arg.splitn(3, ';').collect();
                let degrees = parse_si_num(parts[0])?;
                let minutes = parts.get(1).filter(|s| !s.trim().is_empty())
                    .map(|s| parse_si_num(s)).transpose()?;
                let seconds = parts.get(2).filter(|s| !s.trim().is_empty())
                    .map(|s| parse_si_num(s)).transpose()?;
                return Ok(SiUnitX::AngleDMS { degrees, minutes, seconds });
            } else {
                return Ok(SiUnitX::Angle { value: parse_si_num(arg)? });
            }
        }
        // ── \unit / \si ───────────────────────────────────────────────────
        for prefix in &["\\unit", "\\si"] {
            if let Some(rest) = s.strip_prefix(prefix) {
                let (arg, _) = grab_brace_arg(rest)
                    .ok_or_else(|| anyhow::anyhow!("bad \\unit arg"))?;
                return Ok(SiUnitX::Unit { unit: parse_compound_unit(arg)? });
            }
        }
        // ── \qtyrange ─────────────────────────────────────────────────────
        if let Some(rest) = s.strip_prefix("\\qtyrange") {
            let (a1, rest) = grab_brace_arg(rest)
                .ok_or_else(|| anyhow::anyhow!("bad \\qtyrange arg1"))?;
            let (a2, rest) = grab_brace_arg(rest.trim_start())
                .ok_or_else(|| anyhow::anyhow!("bad \\qtyrange arg2"))?;
            let (u, _) = grab_brace_arg(rest.trim_start())
                .ok_or_else(|| anyhow::anyhow!("bad \\qtyrange unit"))?;
            return Ok(SiUnitX::QuantityRange {
                min: parse_si_num(a1)?,
                max: parse_si_num(a2)?,
                unit: parse_compound_unit(u)?,
            });
        }
        // ── \qtyproduct ───────────────────────────────────────────────────
        if let Some(rest) = s.strip_prefix("\\qtyproduct") {
            let (varg, rest) = grab_brace_arg(rest)
                .ok_or_else(|| anyhow::anyhow!("bad \\qtyproduct arg"))?;
            let (u, _) = grab_brace_arg(rest.trim_start())
                .ok_or_else(|| anyhow::anyhow!("bad \\qtyproduct unit"))?;
            return Ok(SiUnitX::QuantityProduct {
                factors: parse_num_product(varg)?,
                unit: parse_compound_unit(u)?,
            });
        }
        // ── \qtylist ──────────────────────────────────────────────────────
        if let Some(rest) = s.strip_prefix("\\qtylist") {
            let (varg, rest) = grab_brace_arg(rest)
                .ok_or_else(|| anyhow::anyhow!("bad \\qtylist arg"))?;
            let (u, _) = grab_brace_arg(rest.trim_start())
                .ok_or_else(|| anyhow::anyhow!("bad \\qtylist unit"))?;
            return Ok(SiUnitX::QuantityList {
                values: parse_num_semicolon_list(varg)?,
                unit: parse_compound_unit(u)?,
            });
        }
        // ── \qty / \SI ────────────────────────────────────────────────────
        for prefix in &["\\qty", "\\SI"] {
            if let Some(rest) = s.strip_prefix(prefix) {
                let (varg, rest) = grab_brace_arg(rest)
                    .ok_or_else(|| anyhow::anyhow!("bad \\qty value arg"))?;
                let (u, _) = grab_brace_arg(rest.trim_start())
                    .ok_or_else(|| anyhow::anyhow!("bad \\qty unit arg"))?;
                return Ok(SiUnitX::Quantity {
                    value: parse_si_num(varg)?,
                    unit: parse_compound_unit(u)?,
                });
            }
        }
        // ── \complexqty ───────────────────────────────────────────────────
        if let Some(rest) = s.strip_prefix("\\complexqty") {
            let (varg, rest) = grab_brace_arg(rest)
                .ok_or_else(|| anyhow::anyhow!("bad \\complexqty num arg"))?;
            let (u, _) = grab_brace_arg(rest.trim_start())
                .ok_or_else(|| anyhow::anyhow!("bad \\complexqty unit arg"))?;
            let (real, imag) = parse_complex(varg)?;
            return Ok(SiUnitX::ComplexQty { real, imag, unit: parse_compound_unit(u)? });
        }
        // ── \complexnum ───────────────────────────────────────────────────
        if let Some(rest) = s.strip_prefix("\\complexnum") {
            let (arg, _) = grab_brace_arg(rest)
                .ok_or_else(|| anyhow::anyhow!("bad \\complexnum arg"))?;
            let (real, imag) = parse_complex(arg)?;
            return Ok(SiUnitX::ComplexNum { real, imag });
        }

        anyhow::bail!("not a recognised siunitx command: {:?}", s)
    }
}

// ── Parsing primitives ────────────────────────────────────────────────────────

/// Extract the contents of the first `{…}` brace group.  Returns
/// `Some((inner, rest_after_closing_brace))` or `None` if the string does not
/// start with `{`.
fn grab_brace_arg(s: &str) -> Option<(&str, &str)> {
    let s = s.trim_start();
    if !s.starts_with('{') {
        return None;
    }
    let s = &s[1..]; // skip `{`
    let mut depth: usize = 1;
    for (i, c) in s.char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some((&s[..i], &s[i + 1..]));
                }
            }
            _ => {}
        }
    }
    None
}

/// Parse a siunitx number string into `f64`.
///
/// siunitx accepts decimal notation, `e`-notation (`1.5e3`, `1.5e-3`), and
/// locale-agnostic separators.  Rust's built-in `f64::from_str` handles the
/// common cases.  Leading/trailing whitespace is stripped.
fn parse_si_num(s: &str) -> anyhow::Result<f64> {
    let s = s.trim();
    // siunitx uses `e` for the exponent just like Rust.
    s.parse::<f64>().map_err(|_| anyhow::anyhow!("cannot parse {:?} as a number", s))
}

/// Parse a semicolon-separated list of numbers (`"1; 2; 3"`).
fn parse_num_semicolon_list(s: &str) -> anyhow::Result<Vec<f64>> {
    s.split(';')
        .map(|part| parse_si_num(part.trim()))
        .collect()
}

/// Parse a `\numproduct` / `\qtyproduct` argument: values separated by ` x `.
fn parse_num_product(s: &str) -> anyhow::Result<Vec<f64>> {
    s.split('x')
        .map(|part| parse_si_num(part.trim()))
        .collect()
}

/// Parse a complex number expressed as `real+imagi` or `real-imagi`.
///
/// Handles: `1+2i`, `1-2i`, `3i`, `3`, `-1.5+2.3i`.
fn parse_complex(s: &str) -> anyhow::Result<(f64, f64)> {
    let s = s.trim();

    // Strip trailing `i` which marks the imaginary part.
    if let Some(body) = s.strip_suffix('i') {
        // Find the last `+` or `-` that is NOT the very first character
        // (to handle a leading minus on the real part).
        if let Some(pos) = body[1..].rfind(|c| c == '+' || c == '-') {
            let split = pos + 1; // adjust for sub-slice offset
            let real_str = &body[..split];
            let imag_str = &body[split..];
            return Ok((
                parse_si_num(real_str)?,
                parse_si_num(imag_str)?,
            ));
        }
        // No split found → purely imaginary or just a number without separator.
        // If the whole body is parseable it's the imaginary coefficient.
        match parse_si_num(body) {
            Ok(v) => return Ok((0.0, v)),
            Err(_) => {}
        }
    }
    // No trailing `i` → purely real.
    Ok((parse_si_num(s)?, 0.0))
}

// ── Unit string parser ────────────────────────────────────────────────────────

/// Parse a siunitx unit expression (`\kilo\meter\per\second\squared`) into a
/// [`CompoundUnit`].
///
/// The parser is tolerant: unknown/unsupported commands are silently skipped
/// so that user-defined unit macros do not cause a hard failure.
fn parse_compound_unit(s: &str) -> anyhow::Result<CompoundUnit> {
    let s = s.trim();
    let mut factors: Vec<UnitFactor> = Vec::new();
    let mut pending_prefix: Option<SiPrefix> = None;
    let mut next_denominator = false;

    let mut remaining = s;
    while !remaining.is_empty() {
        // Skip to next backslash.
        let Some(pos) = remaining.find('\\') else { break };
        remaining = &remaining[pos + 1..]; // skip `\`

        // Read the command name (ASCII letters only).
        let cmd_end = remaining
            .find(|c: char| !c.is_ascii_alphabetic())
            .unwrap_or(remaining.len());
        let cmd = &remaining[..cmd_end];
        remaining = &remaining[cmd_end..];

        match cmd {
            "per" => {
                next_denominator = true;
            }
            "squared" => {
                if let Some(last) = factors.last_mut() {
                    last.power = if last.power < 0 { -2 } else { 2 };
                }
            }
            "cubed" => {
                if let Some(last) = factors.last_mut() {
                    last.power = if last.power < 0 { -3 } else { 3 };
                }
            }
            "tothe" => {
                if let Some((n_str, rest)) = grab_brace_arg(remaining) {
                    remaining = rest;
                    if let Ok(n) = n_str.trim().parse::<u32>() {
                        if let Some(last) = factors.last_mut() {
                            last.power = if last.power < 0 {
                                -(n as i32)
                            } else {
                                n as i32
                            };
                        }
                    }
                }
            }
            cmd => {
                if let Some(prefix) = prefix_from_latex_cmd(cmd) {
                    pending_prefix = Some(prefix);
                } else if let Some(unit) = unit_from_latex_cmd(cmd) {
                    let power = if next_denominator { -1 } else { 1 };
                    factors.push(UnitFactor {
                        prefix: pending_prefix.take(),
                        unit,
                        power,
                    });
                    next_denominator = false;
                }
                // unknown macro → silently skip
            }
        }
    }

    if factors.is_empty() {
        anyhow::bail!("no recognisable units in {:?}", s);
    }
    Ok(CompoundUnit::new(factors))
}

/// Map a siunitx prefix command name (without leading `\`) to [`SiPrefix`].
fn prefix_from_latex_cmd(cmd: &str) -> Option<SiPrefix> {
    Some(match cmd {
        "quecto" => SiPrefix::Quecto,
        "ronto"  => SiPrefix::Ronto,
        "yocto"  => SiPrefix::Yocto,
        "zepto"  => SiPrefix::Zepto,
        "atto"   => SiPrefix::Atto,
        "femto"  => SiPrefix::Femto,
        "pico"   => SiPrefix::Pico,
        "nano"   => SiPrefix::Nano,
        "micro"  => SiPrefix::Micro,
        "milli"  => SiPrefix::Milli,
        "centi"  => SiPrefix::Centi,
        "deci"   => SiPrefix::Deci,
        "deca"   => SiPrefix::Deca,
        "hecto"  => SiPrefix::Hecto,
        "kilo"   => SiPrefix::Kilo,
        "mega"   => SiPrefix::Mega,
        "giga"   => SiPrefix::Giga,
        "tera"   => SiPrefix::Tera,
        "peta"   => SiPrefix::Peta,
        "exa"    => SiPrefix::Exa,
        "zetta"  => SiPrefix::Zetta,
        "yotta"  => SiPrefix::Yotta,
        "ronna"  => SiPrefix::Ronna,
        "quetta" => SiPrefix::Quetta,
        _ => return None,
    })
}

/// Map a siunitx unit command name (without leading `\`) to [`SIUnit`].
fn unit_from_latex_cmd(cmd: &str) -> Option<SIUnit> {
    Some(match cmd {
        "meter" | "metre"         => SIUnit::Meter,
        "kilogram"                => SIUnit::Kilogram,
        "second"                  => SIUnit::Second,
        "ampere"                  => SIUnit::Ampere,
        "kelvin"                  => SIUnit::Kelvin,
        "mole"                    => SIUnit::Mole,
        "candela"                 => SIUnit::Candela,
        "radian"                  => SIUnit::Radian,
        "steradian"               => SIUnit::Steradian,
        "hertz"                   => SIUnit::Hertz,
        "newton"                  => SIUnit::Newton,
        "pascal"                  => SIUnit::Pascal,
        "joule"                   => SIUnit::Joule,
        "watt"                    => SIUnit::Watt,
        "coulomb"                 => SIUnit::Coulomb,
        "volt"                    => SIUnit::Volt,
        "farad"                   => SIUnit::Farad,
        "ohm"                     => SIUnit::Ohm,
        "siemens"                 => SIUnit::Siemens,
        "weber"                   => SIUnit::Weber,
        "tesla"                   => SIUnit::Tesla,
        "henry"                   => SIUnit::Henry,
        "lumen"                   => SIUnit::Lumen,
        "lux"                     => SIUnit::Lux,
        "becquerel"               => SIUnit::Becquerel,
        "gray"                    => SIUnit::Gray,
        "sievert"                 => SIUnit::Sievert,
        "katal"                   => SIUnit::Katal,
        "minute"                  => SIUnit::Minute,
        "hour"                    => SIUnit::Hour,
        "day"                     => SIUnit::Day,
        "astronomicalunit"        => SIUnit::AstronomicalUnit,
        "degree"                  => SIUnit::Degree,
        "arcminute"               => SIUnit::Arcminute,
        "arcsecond"               => SIUnit::Arcsecond,
        "hectare"                 => SIUnit::Hectare,
        "litre" | "liter"         => SIUnit::Litre,
        "tonne"                   => SIUnit::Tonne,
        "dalton"                  => SIUnit::Dalton,
        "electronvolt"            => SIUnit::Electronvolt,
        "bar"                     => SIUnit::Bar,
        "atm"                     => SIUnit::Atmosphere,
        "degreeCelsius"           => SIUnit::Celsius,
        "atomicmassunit"          => SIUnit::AtomicMassUnit,
        "neper"                   => SIUnit::Neper,
        "bel"                     => SIUnit::Bel,
        "decibel"                 => SIUnit::Decibel,
        "percent"                 => SIUnit::Percent,
        _ => return None,
    })
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── helpers ──────────────────────────────────────────────────────────────

    fn latex(si: &SiUnitX) -> String {
        <SiUnitX as Renderer<Latex, Universal>>::render(si).expect("latex render failed")
    }

    fn html(si: &SiUnitX) -> String {
        <SiUnitX as Renderer<Html, Universal>>::render(si).expect("html render failed")
    }

    fn md(si: &SiUnitX) -> String {
        <SiUnitX as Renderer<Markdown, Universal>>::render(si).expect("markdown render failed")
    }

    fn parse(s: &str) -> SiUnitX {
        s.parse::<SiUnitX>().expect("parse failed")
    }

    fn compound(unit: SIUnit) -> CompoundUnit {
        CompoundUnit::single(unit)
    }

    fn prefixed(prefix: SiPrefix, unit: SIUnit) -> CompoundUnit {
        CompoundUnit::new(vec![UnitFactor { prefix: Some(prefix), unit, power: 1 }])
    }

    fn per(num: SIUnit, den: SIUnit) -> CompoundUnit {
        CompoundUnit::new(vec![
            UnitFactor::new(num),
            UnitFactor { prefix: None, unit: den, power: -1 },
        ])
    }

    // ── SiPrefix ─────────────────────────────────────────────────────────────

    #[test]
    fn prefix_kilo_latex() {
        assert_eq!(SiPrefix::Kilo.to_latex(), "\\kilo");
    }

    #[test]
    fn prefix_micro_symbol() {
        assert_eq!(SiPrefix::Micro.symbol(), "μ");
    }

    #[test]
    fn prefix_milli_exponent() {
        assert_eq!(SiPrefix::Milli.exponent(), -3);
    }

    #[test]
    fn prefix_giga_exponent() {
        assert_eq!(SiPrefix::Giga.exponent(), 9);
    }

    // ── SIUnit ───────────────────────────────────────────────────────────────

    #[test]
    fn unit_meter_latex() {
        assert_eq!(SIUnit::Meter.to_latex(), "\\meter");
    }

    #[test]
    fn unit_kilogram_symbol() {
        assert_eq!(SIUnit::Kilogram.symbol(), "kg");
    }

    #[test]
    fn unit_kelvin_latex() {
        assert_eq!(SIUnit::Kelvin.to_latex(), "\\kelvin");
    }

    // ── UnitFactor ───────────────────────────────────────────────────────────

    #[test]
    fn unit_factor_simple_latex() {
        let f = UnitFactor::new(SIUnit::Meter);
        assert_eq!(f.to_latex_fragment(), "\\meter");
    }

    #[test]
    fn unit_factor_prefixed_latex() {
        let f = UnitFactor { prefix: Some(SiPrefix::Kilo), unit: SIUnit::Meter, power: 1 };
        assert_eq!(f.to_latex_fragment(), "\\kilo\\meter");
    }

    #[test]
    fn unit_factor_squared_latex() {
        let f = UnitFactor { prefix: None, unit: SIUnit::Meter, power: 2 };
        assert_eq!(f.to_latex_fragment(), "\\meter\\squared");
    }

    #[test]
    fn unit_factor_inverse_latex() {
        let f = UnitFactor { prefix: None, unit: SIUnit::Second, power: -1 };
        assert_eq!(f.to_latex_fragment(), "\\per\\second");
    }

    #[test]
    fn unit_factor_symbol_squared() {
        let f = UnitFactor { prefix: None, unit: SIUnit::Meter, power: 2 };
        assert_eq!(f.symbol_str(), "m²");
    }

    #[test]
    fn unit_factor_prefixed_symbol() {
        let f = UnitFactor { prefix: Some(SiPrefix::Kilo), unit: SIUnit::Meter, power: 1 };
        assert_eq!(f.symbol_str(), "km");
    }

    // ── CompoundUnit ─────────────────────────────────────────────────────────

    #[test]
    fn compound_single_latex() {
        let cu = compound(SIUnit::Meter);
        assert_eq!(cu.to_latex_str(), "\\meter");
    }

    #[test]
    fn compound_per_latex() {
        let cu = per(SIUnit::Meter, SIUnit::Second);
        assert_eq!(cu.to_latex_str(), "\\meter\\per\\second");
    }

    #[test]
    fn compound_prefixed_latex() {
        let cu = prefixed(SiPrefix::Kilo, SIUnit::Meter);
        assert_eq!(cu.to_latex_str(), "\\kilo\\meter");
    }

    #[test]
    fn compound_renderer_latex() {
        let cu = compound(SIUnit::Meter);
        let out = <CompoundUnit as Renderer<Latex, Universal>>::render(&cu).unwrap();
        assert_eq!(out, "\\unit{\\meter}");
    }

    #[test]
    fn compound_renderer_html() {
        let cu = compound(SIUnit::Meter);
        let out = <CompoundUnit as Renderer<Html, Universal>>::render(&cu).unwrap();
        assert_eq!(out, "<span class=\"si-unit\">m</span>");
    }

    #[test]
    fn compound_renderer_markdown() {
        let cu = compound(SIUnit::Meter);
        let out = <CompoundUnit as Renderer<Markdown, Universal>>::render(&cu).unwrap();
        assert_eq!(out, "m");
    }

    #[test]
    fn compound_per_symbol() {
        let cu = per(SIUnit::Meter, SIUnit::Second);
        let out = <CompoundUnit as Renderer<Markdown, Universal>>::render(&cu).unwrap();
        assert_eq!(out, "m/s");
    }

    // ── fmt_num ───────────────────────────────────────────────────────────────

    #[test]
    fn fmt_num_integer() {
        assert_eq!(fmt_num(42.0), "42");
    }

    #[test]
    fn fmt_num_fractional() {
        assert_eq!(fmt_num(1.5), "1.5");
    }

    #[test]
    fn fmt_num_negative_whole() {
        assert_eq!(fmt_num(-3.0), "-3");
    }

    // ── SiUnitX: \num ─────────────────────────────────────────────────────────

    #[test]
    fn num_latex() {
        let si = SiUnitX::Number { value: 1234.0 };
        assert_eq!(latex(&si), "\\num{1234}");
    }

    #[test]
    fn num_html() {
        let si = SiUnitX::Number { value: 1234.0 };
        assert_eq!(html(&si), "<span class=\"si-num\">1234</span>");
    }

    #[test]
    fn num_markdown() {
        let si = SiUnitX::Number { value: 1234.0 };
        assert_eq!(md(&si), "1234");
    }

    #[test]
    fn num_parse() {
        let si = parse("\\num{42}");
        assert_eq!(si, SiUnitX::Number { value: 42.0 });
    }

    #[test]
    fn num_parse_fractional() {
        let si = parse("\\num{3.14}");
        assert_eq!(si, SiUnitX::Number { value: 3.14 });
    }

    // ── SiUnitX: \numlist ─────────────────────────────────────────────────────

    #[test]
    fn numlist_latex() {
        let si = SiUnitX::NumberList { values: vec![1.0, 2.0, 3.0] };
        assert_eq!(latex(&si), "\\numlist{1; 2; 3}");
    }

    #[test]
    fn numlist_html() {
        let si = SiUnitX::NumberList { values: vec![1.0, 2.0, 3.0] };
        assert_eq!(html(&si), "<span class=\"si-numlist\">1, 2, 3</span>");
    }

    #[test]
    fn numlist_markdown() {
        let si = SiUnitX::NumberList { values: vec![1.0, 2.0, 3.0] };
        assert_eq!(md(&si), "1, 2, 3");
    }

    #[test]
    fn numlist_parse() {
        let si = parse("\\numlist{1; 2; 3}");
        assert_eq!(si, SiUnitX::NumberList { values: vec![1.0, 2.0, 3.0] });
    }

    // ── SiUnitX: \numrange ────────────────────────────────────────────────────

    #[test]
    fn numrange_latex() {
        let si = SiUnitX::NumberRange { min: 1.0, max: 10.0 };
        assert_eq!(latex(&si), "\\numrange{1}{10}");
    }

    #[test]
    fn numrange_html() {
        let si = SiUnitX::NumberRange { min: 1.0, max: 10.0 };
        assert_eq!(html(&si), "<span class=\"si-numrange\">1&ndash;10</span>");
    }

    #[test]
    fn numrange_markdown() {
        let si = SiUnitX::NumberRange { min: 1.0, max: 10.0 };
        assert_eq!(md(&si), "1–10");
    }

    #[test]
    fn numrange_parse() {
        let si = parse("\\numrange{1}{10}");
        assert_eq!(si, SiUnitX::NumberRange { min: 1.0, max: 10.0 });
    }

    // ── SiUnitX: \numproduct ──────────────────────────────────────────────────

    #[test]
    fn numproduct_latex() {
        let si = SiUnitX::NumberProduct { factors: vec![2.0, 3.0, 4.0] };
        assert_eq!(latex(&si), "\\numproduct{2 x 3 x 4}");
    }

    #[test]
    fn numproduct_html() {
        let si = SiUnitX::NumberProduct { factors: vec![2.0, 3.0] };
        assert_eq!(html(&si), "<span class=\"si-numproduct\">2 &times; 3</span>");
    }

    #[test]
    fn numproduct_markdown() {
        let si = SiUnitX::NumberProduct { factors: vec![2.0, 3.0] };
        assert_eq!(md(&si), "2 × 3");
    }

    #[test]
    fn numproduct_parse() {
        let si = parse("\\numproduct{2 x 3 x 4}");
        assert_eq!(si, SiUnitX::NumberProduct { factors: vec![2.0, 3.0, 4.0] });
    }

    // ── SiUnitX: \ang ────────────────────────────────────────────────────────

    #[test]
    fn ang_decimal_latex() {
        let si = SiUnitX::Angle { value: 45.0 };
        assert_eq!(latex(&si), "\\ang{45}");
    }

    #[test]
    fn ang_decimal_html() {
        let si = SiUnitX::Angle { value: 45.0 };
        assert_eq!(html(&si), "<span class=\"si-ang\">45&deg;</span>");
    }

    #[test]
    fn ang_decimal_markdown() {
        let si = SiUnitX::Angle { value: 45.0 };
        assert_eq!(md(&si), "45°");
    }

    #[test]
    fn ang_decimal_parse() {
        let si = parse("\\ang{45}");
        assert_eq!(si, SiUnitX::Angle { value: 45.0 });
    }

    #[test]
    fn ang_dms_latex() {
        let si = SiUnitX::AngleDMS { degrees: 12.0, minutes: Some(30.0), seconds: Some(15.0) };
        assert_eq!(latex(&si), "\\ang{12;30;15}");
    }

    #[test]
    fn ang_dms_html() {
        let si = SiUnitX::AngleDMS { degrees: 12.0, minutes: Some(30.0), seconds: Some(15.0) };
        assert_eq!(html(&si), "<span class=\"si-ang\">12&deg;30&#8242;15&#8243;</span>");
    }

    #[test]
    fn ang_dms_markdown() {
        let si = SiUnitX::AngleDMS { degrees: 12.0, minutes: Some(30.0), seconds: Some(15.0) };
        assert_eq!(md(&si), "12°30′15″");
    }

    #[test]
    fn ang_dms_parse() {
        let si = parse("\\ang{12;30;15}");
        assert_eq!(si, SiUnitX::AngleDMS { degrees: 12.0, minutes: Some(30.0), seconds: Some(15.0) });
    }

    #[test]
    fn ang_dms_partial_degrees_only() {
        let si = parse("\\ang{45;;}");
        assert_eq!(si, SiUnitX::AngleDMS { degrees: 45.0, minutes: None, seconds: None });
    }

    // ── SiUnitX: \unit ────────────────────────────────────────────────────────

    #[test]
    fn unit_latex() {
        let si = SiUnitX::Unit { unit: compound(SIUnit::Meter) };
        assert_eq!(latex(&si), "\\unit{\\meter}");
    }

    #[test]
    fn unit_html() {
        let si = SiUnitX::Unit { unit: compound(SIUnit::Meter) };
        assert_eq!(html(&si), "<span class=\"si-unit\">m</span>");
    }

    #[test]
    fn unit_markdown() {
        let si = SiUnitX::Unit { unit: compound(SIUnit::Meter) };
        assert_eq!(md(&si), "m");
    }

    #[test]
    fn unit_parse_simple() {
        let si = parse("\\unit{\\meter}");
        assert_eq!(si, SiUnitX::Unit { unit: compound(SIUnit::Meter) });
    }

    #[test]
    fn unit_parse_prefixed() {
        let si = parse("\\unit{\\kilo\\meter}");
        assert_eq!(si, SiUnitX::Unit { unit: prefixed(SiPrefix::Kilo, SIUnit::Meter) });
    }

    #[test]
    fn unit_parse_per() {
        let si = parse("\\unit{\\meter\\per\\second}");
        assert_eq!(si, SiUnitX::Unit { unit: per(SIUnit::Meter, SIUnit::Second) });
    }

    #[test]
    fn si_alias_parse() {
        // \si is an alias for \unit
        let si = parse("\\si{\\meter}");
        assert_eq!(si, SiUnitX::Unit { unit: compound(SIUnit::Meter) });
    }

    // ── SiUnitX: \qty ─────────────────────────────────────────────────────────

    #[test]
    fn qty_latex() {
        let si = SiUnitX::Quantity { value: 9.8, unit: per(SIUnit::Meter, SIUnit::Second) };
        assert_eq!(latex(&si), "\\qty{9.8}{\\meter\\per\\second}");
    }

    #[test]
    fn qty_html() {
        let si = SiUnitX::Quantity { value: 9.8, unit: compound(SIUnit::Meter) };
        assert_eq!(html(&si), "<span class=\"si-qty\">9.8&thinsp;m</span>");
    }

    #[test]
    fn qty_markdown() {
        let si = SiUnitX::Quantity { value: 100.0, unit: prefixed(SiPrefix::Kilo, SIUnit::Meter) };
        assert_eq!(md(&si), "100\u{202F}km");
    }

    #[test]
    fn qty_parse() {
        let si = parse("\\qty{9.8}{\\meter\\per\\second}");
        assert_eq!(si, SiUnitX::Quantity {
            value: 9.8,
            unit: per(SIUnit::Meter, SIUnit::Second),
        });
    }

    #[test]
    fn SI_alias_parse() {
        // \SI is an alias for \qty
        let si = parse("\\SI{42}{\\meter}");
        assert_eq!(si, SiUnitX::Quantity { value: 42.0, unit: compound(SIUnit::Meter) });
    }

    // ── SiUnitX: \qtyrange ────────────────────────────────────────────────────

    #[test]
    fn qtyrange_latex() {
        let si = SiUnitX::QuantityRange { min: 1.0, max: 10.0, unit: compound(SIUnit::Meter) };
        assert_eq!(latex(&si), "\\qtyrange{1}{10}{\\meter}");
    }

    #[test]
    fn qtyrange_html() {
        let si = SiUnitX::QuantityRange { min: 1.0, max: 10.0, unit: compound(SIUnit::Meter) };
        assert_eq!(html(&si), "<span class=\"si-qtyrange\">1&ndash;10&thinsp;m</span>");
    }

    #[test]
    fn qtyrange_markdown() {
        let si = SiUnitX::QuantityRange { min: 1.0, max: 10.0, unit: compound(SIUnit::Meter) };
        assert_eq!(md(&si), "1–10\u{202F}m");
    }

    #[test]
    fn qtyrange_parse() {
        let si = parse("\\qtyrange{1}{10}{\\meter}");
        assert_eq!(si, SiUnitX::QuantityRange { min: 1.0, max: 10.0, unit: compound(SIUnit::Meter) });
    }

    // ── SiUnitX: \qtylist ─────────────────────────────────────────────────────

    #[test]
    fn qtylist_latex() {
        let si = SiUnitX::QuantityList { values: vec![1.0, 2.0, 3.0], unit: compound(SIUnit::Meter) };
        assert_eq!(latex(&si), "\\qtylist{1; 2; 3}{\\meter}");
    }

    #[test]
    fn qtylist_html() {
        let si = SiUnitX::QuantityList { values: vec![1.0, 2.0], unit: compound(SIUnit::Meter) };
        assert_eq!(html(&si), "<span class=\"si-qtylist\">1, 2&thinsp;m</span>");
    }

    #[test]
    fn qtylist_markdown() {
        let si = SiUnitX::QuantityList { values: vec![1.0, 2.0], unit: compound(SIUnit::Meter) };
        assert_eq!(md(&si), "1, 2\u{202F}m");
    }

    #[test]
    fn qtylist_parse() {
        let si = parse("\\qtylist{1; 2; 3}{\\meter}");
        assert_eq!(si, SiUnitX::QuantityList {
            values: vec![1.0, 2.0, 3.0],
            unit: compound(SIUnit::Meter),
        });
    }

    // ── SiUnitX: \qtyproduct ──────────────────────────────────────────────────

    #[test]
    fn qtyproduct_latex() {
        let si = SiUnitX::QuantityProduct { factors: vec![2.0, 3.0], unit: compound(SIUnit::Meter) };
        assert_eq!(latex(&si), "\\qtyproduct{2 x 3}{\\meter}");
    }

    #[test]
    fn qtyproduct_html() {
        let si = SiUnitX::QuantityProduct { factors: vec![2.0, 3.0], unit: compound(SIUnit::Meter) };
        assert_eq!(html(&si), "<span class=\"si-qtyproduct\">2 &times; 3&thinsp;m</span>");
    }

    #[test]
    fn qtyproduct_markdown() {
        let si = SiUnitX::QuantityProduct { factors: vec![2.0, 3.0], unit: compound(SIUnit::Meter) };
        assert_eq!(md(&si), "2 × 3\u{202F}m");
    }

    #[test]
    fn qtyproduct_parse() {
        let si = parse("\\qtyproduct{2 x 3}{\\meter}");
        assert_eq!(si, SiUnitX::QuantityProduct {
            factors: vec![2.0, 3.0],
            unit: compound(SIUnit::Meter),
        });
    }

    // ── SiUnitX: \complexnum ──────────────────────────────────────────────────

    #[test]
    fn complexnum_positive_imag_latex() {
        let si = SiUnitX::ComplexNum { real: 3.0, imag: 4.0 };
        assert_eq!(latex(&si), "\\complexnum{3+4i}");
    }

    #[test]
    fn complexnum_negative_imag_latex() {
        let si = SiUnitX::ComplexNum { real: 1.0, imag: -2.0 };
        assert_eq!(latex(&si), "\\complexnum{1-2i}");
    }

    #[test]
    fn complexnum_html() {
        let si = SiUnitX::ComplexNum { real: 3.0, imag: 4.0 };
        assert_eq!(html(&si), "<span class=\"si-complexnum\">3+4<i>i</i></span>");
    }

    #[test]
    fn complexnum_markdown() {
        let si = SiUnitX::ComplexNum { real: 3.0, imag: 4.0 };
        assert_eq!(md(&si), "3+4i");
    }

    #[test]
    fn complexnum_parse_positive() {
        let si = parse("\\complexnum{3+4i}");
        assert_eq!(si, SiUnitX::ComplexNum { real: 3.0, imag: 4.0 });
    }

    #[test]
    fn complexnum_parse_negative_imag() {
        let si = parse("\\complexnum{1-2i}");
        assert_eq!(si, SiUnitX::ComplexNum { real: 1.0, imag: -2.0 });
    }

    // ── SiUnitX: \complexqty ──────────────────────────────────────────────────

    #[test]
    fn complexqty_latex() {
        let si = SiUnitX::ComplexQty { real: 3.0, imag: 4.0, unit: compound(SIUnit::Ohm) };
        assert_eq!(latex(&si), "\\complexqty{3+4i}{\\ohm}");
    }

    #[test]
    fn complexqty_html() {
        let si = SiUnitX::ComplexQty { real: 3.0, imag: 4.0, unit: compound(SIUnit::Ohm) };
        assert_eq!(html(&si), "<span class=\"si-complexqty\">(3+4<i>i</i>)&thinsp;Ω</span>");
    }

    #[test]
    fn complexqty_markdown() {
        let si = SiUnitX::ComplexQty { real: 3.0, imag: 4.0, unit: compound(SIUnit::Ohm) };
        assert_eq!(md(&si), "(3+4i)\u{202F}Ω");
    }

    #[test]
    fn complexqty_parse() {
        let si = parse("\\complexqty{3+4i}{\\ohm}");
        assert_eq!(si, SiUnitX::ComplexQty { real: 3.0, imag: 4.0, unit: compound(SIUnit::Ohm) });
    }

    // ── is_si_command ─────────────────────────────────────────────────────────

    #[test]
    fn is_si_command_num() {
        assert!(SiUnitX::is_si_command("\\num{42}"));
    }

    #[test]
    fn is_si_command_qty() {
        assert!(SiUnitX::is_si_command("\\qty{9.8}{\\meter\\per\\second}"));
    }

    #[test]
    fn is_si_command_false_for_other() {
        assert!(!SiUnitX::is_si_command("\\frac{1}{2}"));
        assert!(!SiUnitX::is_si_command("x^2"));
    }

    // ── parse errors ─────────────────────────────────────────────────────────

    #[test]
    fn parse_unknown_command_errors() {
        assert!("\\unknown{42}".parse::<SiUnitX>().is_err());
    }

    #[test]
    fn from_latex_delegates_to_parse() {
        let a = SiUnitX::from_latex("\\num{1}").unwrap();
        let b = "\\num{1}".parse::<SiUnitX>().unwrap();
        assert_eq!(a, b);
    }
}
