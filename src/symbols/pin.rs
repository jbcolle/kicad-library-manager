use crate::symbols::property::{
    check_expression_validity, KiCadEffects, KiCadLocation,
};
use crate::symbols::Token::Word;
use crate::symbols::{subdivide_expression, Expression, TryFromExpression};
use anyhow::{bail, Error};
use std::str::FromStr;

#[derive(Clone)]
pub(crate) struct KiCadPinName {
    name: String,
    effects: Option<KiCadEffects>,
}

impl TryFromExpression<KiCadPinName> for KiCadPinName {
    fn try_from_expression(expression: Expression) -> Result<KiCadPinName, Error> {
        check_expression_validity(&expression, "name".to_string())?;
        
        let Some(Word(name)) = expression.get(2) else {
            bail!("No pin name found")
        };
        let subexpressions = subdivide_expression(expression[3..expression.len()].to_owned());

        let mut effects = None;

        for subexpression in subexpressions {
            if let Some(Word(property_name)) = subexpression.get(1) {
                match property_name.as_str() {
                    "effects" => effects = Some(KiCadEffects::try_from_expression(subexpression)?),
                    _ => bail!("Not a valid KiCad pin name property: {property_name}"),
                }
            }
        }

        Ok(KiCadPinName {
            name: name.to_string(),
            effects,
        })
    }
}

#[derive(Clone)]
pub(crate) struct KiCadPinNumber {
    number: String,
    effects: Option<KiCadEffects>,
}

impl TryFromExpression<KiCadPinNumber> for KiCadPinNumber {
    fn try_from_expression(expression: Expression) -> Result<KiCadPinNumber, Error> {
        check_expression_validity(&expression, "number".to_string())?;

        let Some(Word(number)) = expression.get(1) else {
            bail!("No pin number found")
        };
        let subexpressions = subdivide_expression(expression[2..expression.len()].to_owned());

        let mut effects = None;

        for subexpression in subexpressions {
            if let Some(Word(property_name)) = subexpression.get(1) {
                match property_name.as_str() {
                    "effects" => effects = Some(KiCadEffects::try_from_expression(subexpression)?),
                    _ => {
                        bail!("Not a valid KiCad pin number property: {property_name}")
                    }
                }
            }
        }

        Ok(KiCadPinNumber {
            number: number.to_string(),
            effects,
        })
    }
}

#[derive(Copy, Clone)]
pub(crate) enum KiCadPinType {
    Passive,
    PowerIn,
    PowerOut,
    Input,
    Unspecified,
}

impl FromStr for KiCadPinType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "passive" => Ok(Self::Passive),
            "power_in" => Ok(Self::PowerIn),
            "power_out" => Ok(Self::PowerOut),
            "input" => Ok(Self::Input),
            "unspecified" => Ok(Self::Unspecified),
            _ => bail!("Not a valid KiCad pin type: {s}"),
        }
    }
}

#[derive(Copy, Clone)]
pub(crate) enum KiCadPinPolarity {
    Line,
    Inverted,
}

impl FromStr for KiCadPinPolarity {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "line" => Ok(Self::Line),
            "inverted" => Ok(Self::Inverted),
            _ => bail!("Not a valid KiCad pin polarity"),
        }
    }
}

#[derive(Copy, Clone)]
pub(crate) struct KiCadPinLength(f32);

impl TryFromExpression<KiCadPinLength> for KiCadPinLength {
    fn try_from_expression(expression: Expression) -> Result<KiCadPinLength, Error> {
        check_expression_validity(&expression, "length".to_string())?;
        
        let Some(Word(length)) = expression.get(2) else {
            bail!("No pin length found")
        };
        let length = length.parse::<f32>()?;
        Ok(KiCadPinLength(length))
    }
}

#[derive(Clone)]
pub(crate) struct KiCadPin {
    pin_type: KiCadPinType,
    pin_polarity: KiCadPinPolarity,
    location: Option<KiCadLocation>,
    length: Option<KiCadPinLength>,
    name: Option<KiCadPinName>,
    number: Option<KiCadPinNumber>,
}

impl TryFromExpression<KiCadPin> for KiCadPin {
    fn try_from_expression(expression: Expression) -> Result<KiCadPin, Error> {
        check_expression_validity(&expression, "pin".to_string())?;

        let Some(Word(pin_type)) = expression.get(2) else {
            bail!("No pin type found")
        };
        let Some(Word(pin_polarity)) = expression.get(3) else {
            bail!("No pin polarity found")
        };
        let pin_type = KiCadPinType::from_str(pin_type)?;
        let pin_polarity = KiCadPinPolarity::from_str(pin_polarity)?;

        let subexpressions = subdivide_expression(expression[4..expression.len()].to_owned());

        let mut pin_name = None;
        let mut pin_number = None;
        let mut pin_location = None;
        let mut pin_length = None;

        for subexpression in subexpressions {
            if let Some(Word(property_name)) = subexpression.get(1) {
                match property_name.as_str() {
                    "name" => pin_name = Some(KiCadPinName::try_from_expression(subexpression)?),
                    "number" => pin_number = Some(KiCadPinNumber::try_from_expression(subexpression)?),
                    "at" => pin_location = Some(KiCadLocation::try_from_expression(subexpression)?),
                    "length" => pin_length = Some(KiCadPinLength::try_from_expression(subexpression)?),
                    _ => {}
                }
            }
        }
        Ok(KiCadPin {
            pin_type,
            pin_polarity,
            location: pin_location,
            length: pin_length,
            name: pin_name,
            number: pin_number,
        })
    }
}
