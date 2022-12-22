use std::{
    fmt,
    ops::{Div, Mul},
};

use crate::interner::InternedString;

pub(crate) use conversion::{known_compatibilities_by_unit, UNIT_CONVERSION_TABLE};

mod conversion;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub(crate) enum Unit {
    // Absolute units
    /// Pixels
    Px,
    /// Millimeters
    Mm,
    /// Inches
    In,
    /// Centimeters
    Cm,
    /// Quarter-millimeters
    Q,
    /// Points
    Pt,
    /// Picas
    Pc,

    // Font relative units
    /// Font size of the parent element
    Em,
    /// Font size of the root element
    Rem,
    /// Line height of the element
    Lh,
    /// x-height of the element's font
    Ex,
    /// The advance measure (width) of the glyph "0" of the element's font
    Ch,
    /// Represents the "cap height" (nominal height of capital letters) of the element's font
    Cap,
    /// Equal to the used advance measure of the "水" (CJK water ideograph, U+6C34) glyph
    /// found in the font used to render it
    Ic,
    /// Equal to the computed value of the line-height property on the root element
    /// (typically <html>), converted to an absolute length
    Rlh,

    // Viewport relative units
    /// 1% of the viewport's width
    Vw,
    /// 1% of the viewport's height
    Vh,
    /// 1% of the viewport's smaller dimension
    Vmin,
    /// 1% of the viewport's larger dimension
    Vmax,
    /// Equal to 1% of the size of the initial containing block, in the direction of the root
    /// element's inline axis
    Vi,
    /// Equal to 1% of the size of the initial containing block, in the direction of the root
    /// element's block axis
    Vb,

    // Angle units
    /// Represents an angle in degrees. One full circle is 360deg
    Deg,
    /// Represents an angle in gradians. One full circle is 400grad
    Grad,
    /// Represents an angle in radians. One full circle is 2π radians which approximates to 6.283rad
    Rad,
    /// Represents an angle in a number of turns. One full circle is 1turn
    Turn,

    // Time units
    /// Represents a time in seconds
    S,
    /// Represents a time in milliseconds
    Ms,

    // Frequency units
    /// Represents a frequency in hertz
    Hz,
    /// Represents a frequency in kilohertz
    Khz,

    // Resolution units
    /// Represents the number of dots per inch
    Dpi,
    /// Represents the number of dots per centimeter
    Dpcm,
    /// Represents the number of dots per px unit
    Dppx,

    // Other units
    /// Represents a fraction of the available space in the grid container
    Fr,
    Percent,

    /// Unknown unit
    Unknown(InternedString),
    /// Unspecified unit
    None,

    Complex {
        numer: Vec<Unit>,
        denom: Vec<Unit>,
    },
}
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum UnitKind {
    Absolute,
    FontRelative,
    ViewportRelative,
    Angle,
    Time,
    Frequency,
    Resolution,
    Other,
    None,
}

impl Mul<Unit> for Unit {
    type Output = Unit;
    fn mul(self, rhs: Unit) -> Self::Output {
        if self == Unit::None {
            return rhs;
        } else if rhs == Unit::None {
            return self;
        }

        match (self, rhs) {
            (
                Unit::Complex {
                    numer: mut numer1,
                    denom: mut denom1,
                },
                Unit::Complex {
                    numer: mut numer2,
                    denom: mut denom2,
                },
            ) => {
                numer1.append(&mut numer2);
                denom1.append(&mut denom2);

                Unit::Complex {
                    numer: numer1,
                    denom: denom1,
                }
            }
            (
                Unit::Complex {
                    mut numer,
                    mut denom,
                },
                other,
            ) => {
                if let Some(pos_of_other) = denom.iter().position(|denom_unit| denom_unit == &other)
                {
                    denom.remove(pos_of_other);
                } else {
                    numer.push(other);
                }

                if numer.is_empty() && denom.is_empty() {
                    return Unit::None;
                }

                Unit::Complex { numer, denom }
            }
            (other, Unit::Complex { mut numer, denom }) => {
                numer.insert(0, other);
                Unit::Complex { numer, denom }
            }
            (lhs, rhs) => Unit::Complex {
                numer: vec![lhs, rhs],
                denom: Vec::new(),
            },
        }
        .simplify()
    }
}

impl Div<Unit> for Unit {
    type Output = Unit;
    fn div(self, rhs: Unit) -> Self::Output {
        if rhs == Unit::None {
            return self;
        }

        match (self, rhs) {
            (
                Unit::Complex {
                    numer: mut numer1,
                    denom: mut denom1,
                },
                Unit::Complex {
                    numer: mut numer2,
                    denom: mut denom2,
                },
            ) => {
                todo!()
                // numer1.append(&mut numer2);
                // denom1.append(&mut denom2);

                // Unit::Complex {
                //     numer: numer1,
                //     denom: denom1,
                // }
            }
            (
                Unit::Complex {
                    mut numer,
                    mut denom,
                },
                other,
            ) => {
                if let Some(pos_of_other) = numer.iter().position(|numer_unit| numer_unit == &other)
                {
                    numer.remove(pos_of_other);
                } else {
                    denom.push(other);
                }

                if numer.is_empty() && denom.is_empty() {
                    return Unit::None;
                }

                Unit::Complex { numer, denom }
            }
            (
                other,
                Unit::Complex {
                    mut numer,
                    mut denom,
                },
            ) => {
                if let Some(pos_of_other) = numer.iter().position(|numer_unit| numer_unit == &other)
                {
                    numer.remove(pos_of_other);
                } else {
                    denom.insert(0, other);
                }

                if numer.is_empty() && denom.is_empty() {
                    return Unit::None;
                }

                Unit::Complex {
                    numer: denom,
                    denom: numer,
                }
            }
            (Unit::None, rhs) => Unit::Complex {
                numer: Vec::new(),
                denom: vec![rhs],
            },
            (lhs, rhs) => Unit::Complex {
                numer: vec![lhs],
                denom: vec![rhs],
            },
        }
        .simplify()
    }
}

impl Unit {
    fn simplify(self) -> Self {
        match self {
            Unit::Complex { mut numer, denom } if denom.is_empty() && numer.len() == 1 => {
                numer.pop().unwrap()
            }
            _ => self,
        }
    }

    pub fn is_complex(&self) -> bool {
        matches!(self, Unit::Complex { .. })
    }

    pub fn comparable(&self, other: &Unit) -> bool {
        if other == &Unit::None {
            return true;
        }
        match self.kind() {
            UnitKind::FontRelative | UnitKind::ViewportRelative | UnitKind::Other => self == other,
            UnitKind::None => true,
            u => other.kind() == u,
        }
    }

    /// Used internally to determine if two units are comparable or not
    fn kind(&self) -> UnitKind {
        match self {
            Unit::Px | Unit::Mm | Unit::In | Unit::Cm | Unit::Q | Unit::Pt | Unit::Pc => {
                UnitKind::Absolute
            }
            Unit::Em
            | Unit::Rem
            | Unit::Lh
            | Unit::Ex
            | Unit::Ch
            | Unit::Cap
            | Unit::Ic
            | Unit::Rlh => UnitKind::FontRelative,
            Unit::Vw | Unit::Vh | Unit::Vmin | Unit::Vmax | Unit::Vi | Unit::Vb => {
                UnitKind::ViewportRelative
            }
            Unit::Deg | Unit::Grad | Unit::Rad | Unit::Turn => UnitKind::Angle,
            Unit::S | Unit::Ms => UnitKind::Time,
            Unit::Hz | Unit::Khz => UnitKind::Frequency,
            Unit::Dpi | Unit::Dpcm | Unit::Dppx => UnitKind::Resolution,
            Unit::None => UnitKind::None,
            Unit::Fr | Unit::Percent | Unit::Unknown(..) | Unit::Complex { .. } => UnitKind::Other,
        }
    }
}

impl From<String> for Unit {
    fn from(unit: String) -> Self {
        match unit.to_ascii_lowercase().as_str() {
            "px" => Unit::Px,
            "mm" => Unit::Mm,
            "in" => Unit::In,
            "cm" => Unit::Cm,
            "q" => Unit::Q,
            "pt" => Unit::Pt,
            "pc" => Unit::Pc,
            "em" => Unit::Em,
            "rem" => Unit::Rem,
            "lh" => Unit::Lh,
            "%" => Unit::Percent,
            "ex" => Unit::Ex,
            "ch" => Unit::Ch,
            "cap" => Unit::Cap,
            "ic" => Unit::Ic,
            "rlh" => Unit::Rlh,
            "vw" => Unit::Vw,
            "vh" => Unit::Vh,
            "vmin" => Unit::Vmin,
            "vmax" => Unit::Vmax,
            "vi" => Unit::Vi,
            "vb" => Unit::Vb,
            "deg" => Unit::Deg,
            "grad" => Unit::Grad,
            "rad" => Unit::Rad,
            "turn" => Unit::Turn,
            "s" => Unit::S,
            "ms" => Unit::Ms,
            "hz" => Unit::Hz,
            "khz" => Unit::Khz,
            "dpi" => Unit::Dpi,
            "dpcm" => Unit::Dpcm,
            "dppx" => Unit::Dppx,
            "fr" => Unit::Fr,
            _ => Unit::Unknown(InternedString::get_or_intern(unit)),
        }
    }
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Unit::Px => write!(f, "px"),
            Unit::Mm => write!(f, "mm"),
            Unit::In => write!(f, "in"),
            Unit::Cm => write!(f, "cm"),
            Unit::Q => write!(f, "q"),
            Unit::Pt => write!(f, "pt"),
            Unit::Pc => write!(f, "pc"),
            Unit::Em => write!(f, "em"),
            Unit::Rem => write!(f, "rem"),
            Unit::Lh => write!(f, "lh"),
            Unit::Percent => write!(f, "%"),
            Unit::Ex => write!(f, "ex"),
            Unit::Ch => write!(f, "ch"),
            Unit::Cap => write!(f, "cap"),
            Unit::Ic => write!(f, "ic"),
            Unit::Rlh => write!(f, "rlh"),
            Unit::Vw => write!(f, "vw"),
            Unit::Vh => write!(f, "vh"),
            Unit::Vmin => write!(f, "vmin"),
            Unit::Vmax => write!(f, "vmax"),
            Unit::Vi => write!(f, "vi"),
            Unit::Vb => write!(f, "vb"),
            Unit::Deg => write!(f, "deg"),
            Unit::Grad => write!(f, "grad"),
            Unit::Rad => write!(f, "rad"),
            Unit::Turn => write!(f, "turn"),
            Unit::S => write!(f, "s"),
            Unit::Ms => write!(f, "ms"),
            Unit::Hz => write!(f, "Hz"),
            Unit::Khz => write!(f, "kHz"),
            Unit::Dpi => write!(f, "dpi"),
            Unit::Dpcm => write!(f, "dpcm"),
            Unit::Dppx => write!(f, "dppx"),
            Unit::Fr => write!(f, "fr"),
            Unit::Unknown(s) => write!(f, "{}", s),
            Unit::None => Ok(()),
            Unit::Complex { numer, denom } => {
                debug_assert!(numer.len() > 1 || !denom.is_empty());

                let numer_rendered = numer
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<String>>()
                    .join("*");

                let denom_rendered = denom
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<String>>()
                    .join("*");

                if denom.is_empty() {
                    write!(f, "{}", numer_rendered)
                } else if numer.is_empty() && denom.len() == 1 {
                    write!(f, "{}^-1", denom_rendered)
                } else if numer.is_empty() {
                    write!(f, "({})^-1", denom_rendered)
                } else {
                    write!(f, "{}/{}", numer_rendered, denom_rendered)
                }
            }
        }
    }
}
