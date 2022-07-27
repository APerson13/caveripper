use std::{cmp::Ordering, fmt::Display};
use nom::{
    sequence::tuple, 
    character::{
        complete::{multispace1, multispace0, alpha1, alphanumeric1, u32 as nomU32},
    }, 
    branch::alt, bytes::complete::tag, multi::many1, combinator::recognize, IResult
};

use crate::{errors::SearchConditionError, layout::Layout, caveinfo::RoomType};

/// Programmatically defined conditions to search for in a sublevel
#[derive(Clone, Debug)]
pub enum SearchCondition {
    CountEntity{ name: String, relationship: Ordering, amount: usize },
    CountRoomType{ room_type: RoomType, relationship: Ordering, amount: usize },
}

impl SearchCondition {
    pub fn matches(&self, layout: &Layout) -> bool {
        match self { 
            SearchCondition::CountEntity{ name, relationship, amount } => {
                let entity_count = layout.get_spawn_objects()
                    .filter(|entity| entity.name().eq_ignore_ascii_case(name))
                    .count();
                &entity_count.cmp(&amount) == relationship
            },
            SearchCondition::CountRoomType{ room_type, relationship, amount } => {
                let unit_count = layout.map_units.iter()
                    .filter(|unit| &unit.unit.room_type == room_type)
                    .count();
                &unit_count.cmp(&amount) == relationship
            },
        }
    }
}

impl TryFrom<&str> for SearchCondition {
    type Error = SearchConditionError;  // TODO: more specific error type
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let (rest, kind) = condition_kind(value)
            .map_err(|_| SearchConditionError::ParseError)?;
        match kind.to_ascii_lowercase().as_str() {
            "count" => {
                let (_, (name, relationship_char, amount)) = countentity(rest)
                    .map_err(|_| SearchConditionError::ParseError)?;
                let relationship = match relationship_char {
                    "<" => Ordering::Less,
                    "=" => Ordering::Equal,
                    ">" => Ordering::Greater,
                    _ => unreachable!(),
                };
                Ok(SearchCondition::CountEntity{
                    name: name.trim().to_string(), relationship, amount: amount as usize
                })
            },
            "count_unit" => {
                let (_, (room_type_str, relationship_char, amount)) = countentity(rest)
                    .map_err(|_| SearchConditionError::ParseError)?;
                let relationship = match relationship_char {
                    "<" => Ordering::Less,
                    "=" => Ordering::Equal,
                    ">" => Ordering::Greater,
                    _ => unreachable!(),
                };
                let room_type = match room_type_str.to_ascii_lowercase().as_str() {
                    "room" => RoomType::Room,
                    "cap" | "alcove" => RoomType::DeadEnd,
                    "hall" | "hallway" => RoomType::Hallway,
                    _ => return Err(SearchConditionError::InvalidArgument(room_type_str.to_string()))
                };
                Ok(SearchCondition::CountRoomType{
                    room_type, relationship, amount: amount as usize
                })
            },
            _ => panic!("Unrecognized search condition '{}'", kind),
        }
    }
}

impl Display for SearchCondition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self { 
            SearchCondition::CountEntity { name, relationship, amount } => {
                let order_char = match relationship {
                    Ordering::Less => '<',
                    Ordering::Equal => '=',
                    Ordering::Greater => '>'
                };
                write!(f, "count {} {} {}", name, order_char, amount)
            },
            SearchCondition::CountRoomType { room_type, relationship, amount } => {
                let order_char = match relationship {
                    Ordering::Less => '<',
                    Ordering::Equal => '=',
                    Ordering::Greater => '>'
                };
                write!(f, "count_entity {:?} {} {}", room_type, order_char, amount)
            }
        }
    }
}

fn condition_kind(input: &str) -> IResult<&str, &str> {
    let (rest, (_, kind, _)) = tuple((
        multispace0,
        recognize(many1(alt((alpha1, tag("_"), tag("-"))))),
        multispace1,
    ))(input)?;
    Ok((rest, kind))
}

fn countentity(input: &str) -> IResult<&str, (&str, &str, u32)> {
    let (rest, (name, _, relationship_char, _, amount)) = tuple((
        recognize(many1(alt((alphanumeric1, tag("_"), tag("-"))))),
        multispace0,
        alt((tag("<"), tag("="), tag(">"))),
        multispace0,
        nomU32,
    ))(input)?;
    Ok((rest, (name, relationship_char, amount)))
}
