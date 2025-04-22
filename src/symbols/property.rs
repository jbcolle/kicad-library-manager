use crate::symbols::pin::KiCadPin;
use crate::symbols::Token::Word;
use crate::symbols::{subdivide_expression, Expression, Token, TryFromExpression};
use anyhow::{anyhow, bail, Error};
use std::str::FromStr;
use strum::{Display, EnumString};

#[derive(EnumString, Display, Copy, Clone)]
#[strum(serialize_all = "PascalCase")]
pub(crate) enum KiCadPropertyType {
    Reference,
    Value,
    Footprint,
    Datasheet,
    Description,
    #[strum(serialize = "ki_locked")]
    KiLocked,
    #[strum(serialize = "ki_keywords")]
    KiKeywords,
    #[strum(serialize = "ki_fp_filters")]
    KiFpFilters,
    #[strum(serialize = "PARTREV")]
    PartRev,
    #[strum(serialize = "STANDARD")]
    Standard,
    #[strum(serialize = "MAXIMUM_PACKAGE_HEIGHT")]
    MaximumPackageHeight,
    #[strum(serialize = "MANUFACTURER")]
    Manufacturer,
}

#[derive(Clone)]
struct KiCadPropertyId(u32);

impl TryFromExpression<KiCadPropertyId> for KiCadPropertyId {
    fn try_from_expression(expression: Expression) -> Result<KiCadPropertyId, Error> {
        check_expression_validity(&expression, "id".to_string())?;

        if expression.len() < 4 {
            bail!("Property ID expression should have four entries: {expression:?}");
        }
        let Some(Word(id)) = expression.get(2) else { bail!("Property ID does not contain id: {expression:?}") };
        let id = id.parse::<u32>()?;
        Ok(KiCadPropertyId(id))

    }
}

#[derive(Clone)]
pub(crate) struct KiCadProperty {
    property_type: KiCadPropertyType,
    value: String,
    id: Option<KiCadPropertyId>,
    location: Option<KiCadLocation>,
    effects: Option<KiCadEffects>
}

impl TryFromExpression<KiCadProperty> for KiCadProperty {
    fn try_from_expression(expression: Expression) -> Result<KiCadProperty, Error> {
        check_expression_validity(&expression, "property".to_string())?;

        let Some(Word(property_type)) = expression.get(2) else { bail!("Property does not contain type") };
        let Some(Word(value)) = expression.get(3) else { bail!("Property does not contain value") };

        let property_type = KiCadPropertyType::from_str(property_type.as_str())?;

        let mut kicad_property_builder = KiCadPropertyBuilder::new(property_type, value.to_string());

        let subexpressions = subdivide_expression(expression[4..expression.len()].to_owned());

        for expression in subexpressions {
            if let Some(Word(property)) = expression.get(1) {
                let property = property.as_str();
                match property {
                    "id" => {
                        kicad_property_builder.id(KiCadPropertyId::try_from_expression(expression)?);
                    },
                    "at" => {
                        kicad_property_builder.location(KiCadLocation::try_from_expression(expression)?);
                    }
                    "effects" => {
                        kicad_property_builder.effects(KiCadEffects::try_from_expression(expression)?);
                    },
                    _ => {
                        bail!("Not a valid KiCad property: {property}");
                    }
                }
            }
        }
        Ok(kicad_property_builder.build())
    }
}

struct KiCadPropertyBuilder {
    property_type: KiCadPropertyType,
    value: String,
    id: Option<KiCadPropertyId>,
    location: Option<KiCadLocation>,
    effects: Option<KiCadEffects>
}

impl KiCadPropertyBuilder {
    fn new(property_type: KiCadPropertyType, value: String) -> Self {
        Self { property_type, value, id: None, location: None, effects: None }
    }
    fn id(&mut self, id: KiCadPropertyId) -> &mut KiCadPropertyBuilder {
        self.id = Some(id);
        self
    }
    fn location(&mut self, location: KiCadLocation) -> &mut KiCadPropertyBuilder {
        self.location = Some(location);
        self
    }
    fn effects(&mut self, effects: KiCadEffects) -> &mut KiCadPropertyBuilder {
        self.effects = Some(effects);
        self
    }
    fn build(self) -> KiCadProperty {
        KiCadProperty { property_type: self.property_type, value: self.value, id: self.id, location: self.location, effects: self.effects }
    }
}

pub(crate) type KiCadLocation = (f32, f32, f32);

impl TryFromExpression<KiCadLocation> for KiCadLocation {
    fn try_from_expression(expression: Expression) -> Result<KiCadLocation, Error> {
        check_expression_validity(&expression, "at".to_string())?;

        if expression.len() < 5 {
            bail!("Location expression should have five entries: {expression:?}");
        }
        let Some(Word(x)) = expression.get(2) else { bail!("Location does not contain x") };
        let Some(Word(y)) = expression.get(3) else { bail!("Location does not contain y") };
        let Some(Word(z)) = expression.get(4) else { bail!("Location does not contain z") };

        let x = x.parse::<f32>()?;
        let y = y.parse::<f32>()?;
        let z = z.parse::<f32>()?;

        Ok((x, y, z))
    }
}

#[derive(Copy, Clone)]
pub(crate) struct KiCadFontSize {
    width: f32,
    height: f32,
}

impl TryFromExpression<KiCadFontSize> for KiCadFontSize {
    fn try_from_expression(expression: Expression) -> Result<KiCadFontSize, Error> {
        check_expression_validity(&expression, "size".to_string())?;

        if expression.len() != 5 {
            bail!("Font size expression should have four entries: {expression:?}");
        }
        let Some(Word(width)) = expression.get(2) else { bail!("Font size does not contain width") };
        let Some(Word(height)) = expression.get(3) else { bail!("Font size does not contain height") };

        let width = width.parse::<f32>()?;
        let height = height.parse::<f32>()?;

        Ok(KiCadFontSize { width, height })
    }
}

#[derive(Copy, Clone)]
pub(crate) struct KiCadFont {
    font_size: Option<KiCadFontSize>,
    bold: bool,
    italic: bool,
    subscript: bool,
    superscript: bool,
    overbar: bool,
    underline: bool,
}

impl TryFromExpression<KiCadFont> for KiCadFont {
    fn try_from_expression(expression: Expression) -> Result<KiCadFont, Error> {
        check_expression_validity(&expression, "font".to_string())?;

        let subexpressions = subdivide_expression(expression[2..expression.len()].to_owned());

        let mut font_size = None;
        let mut bold = false;
        let mut italic = false;
        let mut subscript = false;
        let mut superscript = false;
        let mut overbar = false;
        let mut underline = false;

        for expression in subexpressions {
            if let Some(Word(property)) = expression.get(1) {
                let property = property.as_str();
                match property {
                    "size" => {
                        font_size = Some(KiCadFontSize::try_from_expression(expression)?);
                    },
                    "bold" => {
                        bold = true;
                    },
                    "italic" => {
                        italic = true;
                    },
                    "subscript" => {
                        subscript = true;
                    },
                    "superscript" => {
                        superscript = true;
                    },
                    "overbar" => {
                        overbar = true;
                    },
                    "underline" => {
                        underline = true;
                    }
                    _ => {
                        bail!("Not a valid KiCad font property: {property}");
                    }
                }
            }
        }

        Ok(Self { font_size, bold, italic, subscript, superscript, overbar, underline })
    }
}

#[derive(Copy, Clone)]
pub(crate) enum KiCadEffectsJustify {
    Bottom,
    Top,
    Left,
    Right,
}

#[derive(Clone)]
pub(crate) struct KiCadEffects {
    font: Option<KiCadFont>,
    hide: bool,
    justify: Vec<KiCadEffectsJustify>,
}

impl TryFromExpression<KiCadEffects> for KiCadEffects {
    fn try_from_expression(expression: Expression) -> Result<KiCadEffects, Error> {
        check_expression_validity(&expression, "effects".to_string())?;

        let subexpressions = subdivide_expression(expression[2..expression.len()].to_owned());

        let mut font = None;
        let mut justify = vec![];
        let mut hide = false;
        for expression in subexpressions {
            if let Some(Word(property)) = expression.get(1) {
                let property = property.as_str();
                match property {
                    "font" => {
                        font = Some(KiCadFont::try_from_expression(expression)?);
                    },
                    "justify" => {
                        if expression.len() < 3 {
                            bail!("Justify does not contain value")
                        }
                        for i in 2..(expression.len() - 1) {
                            let Some(Word(justify_value)) = expression.get(i) else { bail!("Justify does not contain value") };
                            let justify_value = justify_value.as_str();
                            match justify_value {
                                "bottom" => justify.push(KiCadEffectsJustify::Bottom),
                                "top" => justify.push(KiCadEffectsJustify::Top),
                                "left" => justify.push(KiCadEffectsJustify::Left),
                                "right" => justify.push(KiCadEffectsJustify::Right),
                                _ => bail!("Not a valid KiCad effects justify value: {justify_value}"),
                            }
                        }
                    },
                    "hide" => {
                        hide = true;
                    }
                    _ => {
                        bail!("Not a valid KiCad effects property: {property}");
                    }
                }
            }
        }

        Ok(Self { font, hide, justify })
    }
}

#[derive(Clone)]
enum KiCadSingleValueProperty {
    Offset(f32),
    InBom(bool),
    OnBoard(bool),
    ExcludeFromSim(bool),
}

fn try_parse_string_to_bool(value: &str) -> Result<bool, anyhow::Error> {
    match value {
        "yes" => Ok(true),
        "no" => Ok(false),
        _ => bail!("Boolean value not yes or no: {value}")
    }
}

impl TryFromExpression<KiCadSingleValueProperty> for KiCadSingleValueProperty {
    fn try_from_expression(expression: Expression) -> Result<KiCadSingleValueProperty, Error> {
        let Token::Word(prop) = get_expression_first_value(&expression)? else {
            bail!("Expression's second Token is not a word: {expression:?}")
        };
        let Word(value) = expression.get(2).ok_or(anyhow!("Could not get expression second value"))? else { bail!("Expression's second value not a word") };
        
        Ok(match prop.as_str() { 
            "offset" => Self::Offset(value.parse::<f32>()?),
            "in_bom" => Self::InBom(try_parse_string_to_bool(&value)?),
            "on_board" => Self::OnBoard(try_parse_string_to_bool(&value)?),
            "exclude_from_sim" => Self::ExcludeFromSim(try_parse_string_to_bool(&value)?),
            _ => bail!("Not a valid option for KiCadSingleValueProperty: {prop}, {value}"),
        })
        
    }
}

#[derive(Clone)]
pub(crate) struct Offset(f32);

impl TryFromExpression<Offset> for Offset {
    fn try_from_expression(expression: Expression) -> Result<Offset, Error> {
        check_expression_validity(&expression, "offset".to_string())?;
        let Some(Word(offset)) = expression.get(2) else {
            bail!("Offset does not contain value")
        };
        Ok(Self(offset.parse::<f32>()?))
    }
}

#[derive(Clone)]
pub(crate) struct KiCadPinNames {
    offset: Offset,
}

impl TryFromExpression<KiCadPinNames> for KiCadPinNames {
    fn try_from_expression(expression: Expression) -> Result<KiCadPinNames, Error> {
        check_expression_validity(&expression, "pin_names".to_string())?;

        let subexpression = subdivide_expression(expression[2..expression.len()].to_owned());

        if subexpression.len() != 1 {
            unimplemented!()
        }
        let offset = Offset::try_from_expression(subexpression[0].to_owned())?;

        Ok(Self { offset })
    }
}

#[derive(Copy, Clone)]
pub(crate) enum KiCadStrokeType {
    Default,
}

impl FromStr for KiCadStrokeType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "default" => Ok(KiCadStrokeType::Default),
            _ => bail!("Not a valid KiCad stroke type: {s}")
        }
    }
}

#[derive(Copy, Clone)]
pub(crate) struct KiCadStroke {
    width: Option<f32>,
    stroke_type: Option<KiCadStrokeType>,
}

impl TryFromExpression<KiCadStroke> for KiCadStroke {
    fn try_from_expression(expression: Expression) -> Result<KiCadStroke, Error> {
        check_expression_validity(&expression, "stroke".to_string())?;

        let subexpressions = subdivide_expression(expression[2..expression.len()].to_owned());
        let mut width = None;
        let mut stroke_type = None;
        
        for expression in subexpressions {
            if let Some(Word(property)) = expression.get(1) {
                let property = property.as_str();
                match property {
                    "width" => {
                        let Some(Word(width_value)) = expression.get(2) else { bail!("Stroke does not contain width") };
                        width = Some(width_value.parse::<f32>()?);
                    },
                    "type" => {
                        let Some(Word(stroke_type_value)) = expression.get(2) else { bail!("Stroke does not contain type") };
                        stroke_type = Some(KiCadStrokeType::from_str(stroke_type_value.as_str())?);
                    },
                    _ => {
                        bail!("Not a valid KiCad stroke property: {property}");
                    }
                }
            }
        }
        Ok(Self { width, stroke_type })
    }
}

#[derive(Copy, Clone)]
pub(crate) enum KiCadFillType {
    Background,
    Outline,
    None,
}

impl FromStr for KiCadFillType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "background" => Ok(KiCadFillType::Background),
            "outline" => Ok(KiCadFillType::Outline),
            "none" => Ok(KiCadFillType::None),
            _ => bail!("Not a valid KiCad fill type: {s}")
        }
    }
}

#[derive(Copy, Clone)]
pub(crate) struct KiCadFill {
    fill_type: Option<KiCadFillType>,
}

impl TryFromExpression<KiCadFill> for KiCadFill {
    fn try_from_expression(expression: Expression) -> Result<KiCadFill, Error> {
        check_expression_validity(&expression, "fill".to_string())?;
        
        let subexpressions = subdivide_expression(expression[2..expression.len()].to_owned());
        let mut fill_type = None;
        
        for expression in subexpressions {
            if let Some(Word(property)) = expression.get(1) {
                let property = property.as_str();
                match property {
                    "type" => {
                        let Some(Word(fill_type_value)) = expression.get(2) else { bail!("Fill does not contain type") };
                        fill_type = Some(KiCadFillType::from_str(fill_type_value.as_str())?);
                    },
                    _ => {
                        bail!("Not a valid KiCad fill property: {property}");
                    }
                }
            }
        }

        Ok(Self { fill_type })
    }
}

#[derive(Copy, Clone)]
pub(crate) struct KiCad2DPoint {
    x: f32,
    y: f32,
}

#[derive(Copy, Clone)]
pub(crate) struct KiCadXY(KiCad2DPoint);

type KiCadPolylinePts = Vec<KiCadXY>;

impl TryFromExpression<KiCadPolylinePts> for KiCadPolylinePts {
    fn try_from_expression(expression: Expression) -> Result<KiCadPolylinePts, Error> {
        check_expression_validity(&expression, "pts".to_string())?;

        let subexpressions = subdivide_expression(expression[2..expression.len()].to_owned());

        let mut pts = vec![];

        for expression in subexpressions {
            if let Some(Word(property)) = expression.get(1) {
                let property = property.as_str();
                match property {
                    "xy" => {
                        let Some(Word(x)) = expression.get(2) else {
                            bail!("Polyline does not contain x")
                        };
                        let Some(Word(y)) = expression.get(3) else {
                            bail!("Polyline does not contain y")
                        };
                        pts.push(KiCadXY(KiCad2DPoint { x: x.parse::<f32>()?, y: y.parse::<f32>()? }));
                    },
                    _ => {
                        bail!("Not a valid KiCad polyline pts property: {property}");
                    }
                }
            }
        }

        Ok(pts)
    }
}

#[derive(Clone)]
pub(crate) struct KiCadPolyline {
    pts: Vec<KiCadXY>,
    stroke: Option<KiCadStroke>,
    fill: Option<KiCadFill>,
}

impl TryFromExpression<KiCadPolyline> for KiCadPolyline {
    fn try_from_expression(expression: Expression) -> Result<KiCadPolyline, Error> {
        check_expression_validity(&expression, "polyline".to_string())?;

        let subexpressions = subdivide_expression(expression[2..expression.len()].to_owned());

        let mut pts = vec![];
        let mut stroke = None;
        let mut fill = None;

        for expression in subexpressions {
            if let Some(Word(property)) = expression.get(1) {
                let property = property.as_str();
                match property {
                    "pts" => {
                        pts = KiCadPolylinePts::try_from_expression(expression)?
                    },
                    "stroke" => {
                        stroke = Some(KiCadStroke::try_from_expression(expression)?);
                    },
                    "fill" => {
                        fill = Some(KiCadFill::try_from_expression(expression)?);
                    },
                    _ => {
                        bail!("Not a valid KiCad polyline property: {property}");
                    }
                }
            }
        }

        Ok(Self { pts, stroke, fill })
    }
}

#[derive(Clone)]
pub(crate) struct KiCadText {
    text: String,
    location: KiCadLocation,
    effects: Option<KiCadEffects>,
}

impl TryFromExpression<KiCadText> for KiCadText {
    fn try_from_expression(expression: Expression) -> Result<KiCadText, Error> {
        check_expression_validity(&expression, "text".to_string())?;

        let Some(Word(text)) = expression.get(2) else { bail!("Text does not contain text") };

        let subexpressions = subdivide_expression(expression[3..expression.len()].to_owned());

        let mut location = None;
        let mut effects = None;

        for expression in subexpressions {
            if let Some(Word(property)) = expression.get(1) {
                let property = property.as_str();
                match property {
                    "effects" => {
                        effects = Some(KiCadEffects::try_from_expression(expression)?);
                    },
                    "at" => {
                        location = Some(KiCadLocation::try_from_expression(expression)?);
                    },
                    _ => {
                        bail!("Not a valid KiCad text property: {property}");
                    }
                }
            }
        }
        let location = location.ok_or(anyhow!("Text does not contain location"))?;
        Ok(Self { text: text.to_string(), location, effects })
    }
}

#[derive(Clone)]
pub(crate) struct KiCadSymbol {
    name: String,
    pin_names: Option<KiCadPinNames>,
    exclude_from_sim: Option<KiCadSingleValueProperty>,
    in_bom: Option<KiCadSingleValueProperty>,
    on_board: Option<KiCadSingleValueProperty>,
    properties: Vec<KiCadProperty>,
    sub_symbols: Vec<KiCadSubSymbol>,
}

pub(crate) fn check_expression_validity(
    expression: &Expression,
    property: String,
) -> Result<(), anyhow::Error> {
    if expression.len() < 2 {
        bail!("Expression smaller than two: {expression:?}");
    }
    if !(expression.first() == Some(&Token::OpenParen)
        && expression.get(1) == Some(&Word(property)))
    {
        bail!("Not a valid KiCad symbol: {expression:?}")
    }
    Ok(())
}

fn get_expression_first_value(expression: &Expression) -> Result<Token, anyhow::Error> {
    if expression.len() < 2 {
        bail!("Expression smaller than two: {expression:?}");
    }
    if expression.first() != Some(&Token::OpenParen) {
        bail!("Expression does not start with opening parenthesis")
    }
    Ok(expression[1].to_owned())
}

impl TryFromExpression<KiCadSymbol> for KiCadSymbol {
    fn try_from_expression(expression: Expression) -> Result<KiCadSymbol, Error> {
        check_expression_validity(&expression, "symbol".to_string())?;

        let Word(name) = &expression[2] else {
            bail!("Symbol has no name")
        };

        let new_expression = Expression::from(&expression[3..expression.len()]);

        let subexpressions = subdivide_expression(new_expression);
        let mut kicad_symbol_builder = KiCadSymbolBuilder::new(name.to_string());

        for expression in subexpressions {
            
            if let Some(Word(value)) = expression.get(1) {
                let value = value.as_str();
                match value {
                    "pin_names" => {
                        kicad_symbol_builder.pin_names(KiCadPinNames::try_from_expression(expression)?);
                    },
                    "exclude_from_sim" => {
                        kicad_symbol_builder.exclude_from_sim(KiCadSingleValueProperty::try_from_expression(expression)?);
                    },
                    "in_bom" => {
                        kicad_symbol_builder.in_bom(KiCadSingleValueProperty::try_from_expression(expression)?);
                    },
                    "on_board" => {
                        kicad_symbol_builder.on_board(KiCadSingleValueProperty::try_from_expression(expression)?);
                    },
                    "property" => {
                        kicad_symbol_builder.add_property(KiCadProperty::try_from_expression(expression)?);
                    },
                    "symbol" => {
                        kicad_symbol_builder.add_sub_symbol(KiCadSubSymbol::try_from_expression(expression)?);
                    },
                    _ => {
                        bail!("Not a valid KiCad symbol property: {value}");
                    }
                }
            }
        }

        Ok(kicad_symbol_builder.build())
    }
}

struct KiCadSymbolBuilder {
    name: String,
    pin_names: Option<KiCadPinNames>,
    exclude_from_sim: Option<KiCadSingleValueProperty>,
    in_bom: Option<KiCadSingleValueProperty>,
    on_board: Option<KiCadSingleValueProperty>,
    properties: Vec<KiCadProperty>,
    sub_symbols: Vec<KiCadSubSymbol>,
}

impl KiCadSymbolBuilder {
    fn new(name: String) -> Self {
        Self {name, pin_names: None, exclude_from_sim: None, in_bom: None, on_board: None, properties: vec![], sub_symbols: vec![] }
    }
    fn pin_names(&mut self, pin_names: KiCadPinNames) -> &mut KiCadSymbolBuilder {
        self.pin_names = Some(pin_names);
        self
    }
    fn exclude_from_sim(&mut self, exclude_from_sim: KiCadSingleValueProperty) -> &mut KiCadSymbolBuilder {
        self.exclude_from_sim = Some(exclude_from_sim);
        self
    }
    fn in_bom(&mut self, in_bom: KiCadSingleValueProperty) -> &mut KiCadSymbolBuilder {
        self.in_bom = Some(in_bom);
        self
    }
    fn on_board(&mut self, on_board: KiCadSingleValueProperty) -> &mut KiCadSymbolBuilder {
        self.on_board = Some(on_board);
        self
    }
    fn add_property(&mut self, property: KiCadProperty) -> &mut KiCadSymbolBuilder {
        self.properties.push(property);
        self
    }
    fn add_sub_symbol(&mut self, sub_symbol: KiCadSubSymbol) -> &mut KiCadSymbolBuilder {
        self.sub_symbols.push(sub_symbol);
        self
    }
    fn build(self) -> KiCadSymbol {
        KiCadSymbol {
            name: self.name,
            pin_names: self.pin_names,
            exclude_from_sim: self.exclude_from_sim,
            in_bom: self.in_bom,
            on_board: self.on_board,
            properties: self.properties,
            sub_symbols: self.sub_symbols
        }
    }
}

#[derive(Clone)]
pub(crate) struct KiCadSubSymbol {
    polylines: Vec<KiCadPolyline>,
    texts: Vec<KiCadText>,
    pins: Vec<KiCadPin>,
}

impl TryFromExpression<KiCadSubSymbol> for KiCadSubSymbol {
    fn try_from_expression(expression: Expression) -> Result<KiCadSubSymbol, Error> {
        check_expression_validity(&expression, "symbol".to_string())?;
        let subexpressions = subdivide_expression(expression[2..expression.len()].to_owned());

        let mut polylines = vec![];
        let mut texts = vec![];
        let mut pins = vec![];

        for expression in subexpressions {
            if let Some(Word(value)) = expression.get(1) {
                let value = value.as_str();
                match value {
                    "polyline" => {
                        polylines.push(KiCadPolyline::try_from_expression(expression)?);
                    },
                    "text" => {
                        texts.push(KiCadText::try_from_expression(expression)?);
                    },
                    "pin" => {
                        pins.push(KiCadPin::try_from_expression(expression)?);
                    },
                    _ => {
                        bail!("Not a valid KiCad sub symbol property: {value}");
                    }
                }
            }
        }
        Ok(Self { polylines, texts, pins })
    }
}