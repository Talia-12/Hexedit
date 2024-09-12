use std::fmt::Display;
use std::slice::SliceIndex;
use std::{fmt, ops::Index};
use std::collections::HashSet;

use itertools::{Either, Either::Left, Either::Right};

use crate::hex_pattern::HexPattern;

#[derive(Clone)]
pub enum Iota {
	Pattern(HexPattern),
	// None is an unknown double.
	Double(Option<f64>),
	/// the left case is a known vector, the right case is an unknown vector with the boolean representing whether the vec is guaranteed in range.
	Vec(Either<(f64, f64, f64), bool>),
	Widget,
	List(IotaList),
	Entity(IotaEntity)
}

impl From<f64> for Iota {
	fn from(d: f64) -> Self {
		Iota::Double(Some(d))
	}
}

impl From<Option<f64>> for Iota {
	fn from(d: Option<f64>) -> Self {
		Iota::Double(d)
	}
}

impl From<(f64, f64, f64)> for Iota {
	fn from(vec: (f64, f64, f64)) -> Self {
		Iota::Vec(Left(vec))
	}
}

impl From<Either<(f64, f64, f64), bool>> for Iota {
	fn from(vec: Either<(f64, f64, f64), bool>) -> Self {
		Iota::Vec(vec)
	}
}

/// The left case is a known vector of iotas, the right how many elements the list could have.
#[derive(Clone)]
pub struct IotaList(Either<Vec<Iota>, Option<usize>>);

impl IotaList {
	fn new(list: Either<Vec<Iota>, Option<usize>>) -> IotaList { IotaList(list) }
}

#[derive(Clone)]
pub struct IotaEntity {
	name: String,
	uuid: String,
	guaranteed_types: HashSet<EntityType>,
	possible_types: HashSet<EntityType>,
	guaranteed_in_range: bool
}

impl IotaEntity {
	pub fn new (name: &str) -> EntityBuilder {
		EntityBuilder { name: name.to_string(), uuid: name.to_string(), guaranteed_types: None, possible_types: None, guaranteed_in_range: None }
	}

	/// returns true if adding this guaranteed type causes no exclusivity issues, false otherwise.
	fn add_guarenteed(&mut self, guaranteed: EntityType) -> bool {
		if self.guaranteed_types.iter().any(|e_type| e_type.mutually_exclusive(&guaranteed)) {
			return false;
		}

		self.possible_types.retain(|e_type| !e_type.mutually_exclusive(&guaranteed));
		self.guaranteed_types.insert(guaranteed);

		return true;
	}

	/// returns true if removing this possible type causes no exclusivity issues, false otherwise
	fn remove_possible(&mut self, possible: EntityType) -> bool {
		if self.guaranteed_types.contains(&possible) {
			return false;
		}

		self.possible_types.remove(&possible);

		return true;
	}
}

impl From<&mut EntityBuilder> for IotaEntity {
	fn from(builder: &mut EntityBuilder) -> Self {
		IotaEntity {
			name: builder.name.clone(),
			uuid: builder.uuid.clone(),
			guaranteed_types: builder.guaranteed_types.clone().unwrap_or(HashSet::new()),
			possible_types: builder.possible_types.clone().unwrap_or(HashSet::from_iter([EntityType::Animal, EntityType::Monster, EntityType::Item, EntityType::Player, EntityType::Living])),
			guaranteed_in_range: builder.guaranteed_in_range.unwrap_or(true)
		}
	}
}

pub struct EntityBuilder {
	name: String,
	uuid: String,
	guaranteed_types: Option<HashSet<EntityType>>,
	possible_types: Option<HashSet<EntityType>>,
	guaranteed_in_range: Option<bool>
}

impl EntityBuilder {
	pub fn name(&mut self, name: &str) -> &mut Self {
		self.name = name.to_string();
		self
	}

	pub fn uuid(&mut self, uuid: &str) -> &mut Self {
		self.uuid = uuid.to_string();
		self
	}

	pub fn add_guaranteed(&mut self, guaranteed: EntityType) -> &mut Self {
		if let Some(guaranteed_types) = &mut self.guaranteed_types {
    	guaranteed_types.insert(guaranteed);
		} else {
			self.guaranteed_types = Some(HashSet::new());
		}

		self
	}

	pub fn remove_possible(&mut self, possible: EntityType) -> &mut Self {
		if let Some(possible_types) = &mut self.possible_types {
    	possible_types.remove(&possible);
		} else {
			let mut possible_types = HashSet::from_iter([EntityType::Animal, EntityType::Monster, EntityType::Item, EntityType::Player, EntityType::Living]);
			possible_types.remove(&possible);
			self.possible_types = Some(possible_types);
		}

		self
	}

	pub fn set_guaranteed_in_range(&mut self, guaranteed_in_range: bool) -> &mut Self {
		self.guaranteed_in_range = Some(guaranteed_in_range);
		
		self
	}
}


#[derive(PartialEq, Eq, Hash, Clone)]
pub enum EntityType {
	Animal,
	Monster,
	Item,
	Player,
	Living
}

impl EntityType {
	fn mutually_exclusive(&self, other: &EntityType) -> bool {
		match &self {
			EntityType::Animal => *other == EntityType::Monster || *other == EntityType::Player || *other == EntityType::Item,
			EntityType::Monster => *other == EntityType::Animal || *other == EntityType::Player || *other == EntityType::Item,
			EntityType::Item => *other != *self,
			EntityType::Player => *other == EntityType::Animal || *other == EntityType::Monster || *other == EntityType::Item,
			EntityType::Living => *other == EntityType::Item,
		}
	}
}

impl fmt::Display for Iota {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match &self {
			Iota::Pattern(pattern) => {
				write!(f, "HexPattern({}, {:?})", pattern.pattern_vec.iter().fold("".to_string(), |acc, dir| {acc + &format!("{dir:?}")}), pattern.start_dir)
			},
			Iota::Double(d) => if let Some(d) = d { write!(f, "{}", d) } else { write!(f, "UNKOWN") },
			Iota::Vec(vec) => {
				match vec {
						Left(vec) => write!(f, "({}, {}, {})", vec.0, vec.1, vec.2),
						Right(within_range) => write!(f, "(UNKNOWN, UNKNOWN, UNKNOWN ; guaranteed in range: {})", within_range),
				}
			}
			Iota::Widget => write!(f, "Null"),
			Iota::List(iotas) => write!(f, "{}", iotas),
			Iota::Entity(entity) => write!(f, "{}", entity.name),
		}
	}
}

impl fmt::Display for IotaList {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match &self.0 {
    	Left(list) => {
				if list.len() == 0 {
					return write!(f, "[]")
				}
				if list.len() == 1 {
					return write!(f, "[{}]", list[0].to_string())
				}

				let mut comma_separated = String::new();

				list[..list.len() - 1].into_iter().for_each(|iota| {
					comma_separated.push_str(&iota.to_string());
					comma_separated.push_str(", ");
				});

				comma_separated.push_str(&list[list.len() - 1].to_string());
				write!(f, "[{}]", comma_separated)
			},
    	Right(length) => write!(f, "[UNKOWN, len={}]", length.map_or("UNKOWN".to_string(), |length| length.to_string())),
		}
	}
}

/// Stores all the possible stacks that could have been reached at this point.
/// each element of stacks is a possible stack, with the result being Ok if no mishaps have occurred, and being Err if one has.
/// The Vec<Iota> in the Pair is the stack, and the Option<Iota> is the 
#[derive(Default)]
pub struct StackHolder(Vec<Result<StackState, ActionError>>);

impl StackHolder {
	fn iter(&self) -> std::slice::Iter<'_, Result<StackState, ActionError>> { self.0.iter() }
	fn append(&mut self, result: &mut StackHolder) { self.0.append(&mut result.0) }
	fn len(&self) -> usize { self.0.len() }

	pub fn apply_action(&mut self, action: Box<dyn Action>) {
		let mut result_stacks: StackHolder = StackHolder::default();

		for stack in self.iter() {
			if let Ok(stack) = stack { result_stacks.append(&mut action.apply(stack)) }
		}
	}

	pub fn single_state(state: StackState) -> StackHolder { StackHolder(vec![Ok(state)]) }
	pub fn single(result: Result<StackState, ActionError>) -> StackHolder { StackHolder(vec![result]) }
	pub fn new(results: Vec<Result<StackState, ActionError>>) -> StackHolder { StackHolder(results) }
}

#[derive(Default)]
pub struct StackState {
	stack: Vec<Iota>,
	ravenmind: Option<Iota>
}

impl Display for StackState {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		if self.stack.len() == 0 {
			return write!(f, "")
		} if self.stack.len() == 1 {
			return write!(f, "{}", self.stack[0]);
		}

		let mut newline_separated = String::new();
		
		self.stack[..self.stack.len() - 1].into_iter().for_each(|iota| {
			newline_separated.push_str(&iota.to_string());
			newline_separated.push_str("\n");
		});
		let last_element = self.stack.last().map_or("".to_string(), |iota| iota.to_string());
		newline_separated.push_str(&last_element);

		write!(f, "{}", newline_separated)
	}
}

impl StackState {
	pub fn new(stack: Vec<Iota>, ravenmind: Option<Iota>) -> StackState { StackState { stack, ravenmind } }
}

#[derive(Clone, Copy)]
pub enum ActionError {
	OutOfBounds,
	StackTooSmall,
	DivByZero,
	InvalidType
}

pub trait Action {
	/// Takes in a list of vectors representing one possible stack at the point the action is called,
	/// and returns a vector of possible resulting stacks.
	fn apply(&self, iotas: &StackState) -> StackHolder;
}

pub trait ConstLenAction {
	fn len() -> usize;
	
	// takes in an array of vectors representing one possible input the ConstLenAction could receive,
	// and returns a vec of possible outputs of the action.
	fn apply(&self, iotas: &[Iota]) -> Vec<Result<Vec<Iota>, ActionError>>;
}

pub trait FixedArgsAction<Args> {
	fn apply(&self, iotas: Args) -> Vec<Result<Vec<Iota>, ActionError>>;
}

impl <T> Action for T where T: ConstLenAction {
	fn apply(&self, stack_state: &StackState) -> StackHolder {
		let len = T::len();
		let stack_len = stack_state.stack.len();

		if stack_len < len {
			return StackHolder::single(Err(ActionError::StackTooSmall));
		}

		StackHolder::new(
			Vec::from_iter(self.apply(&stack_state.stack[stack_len - len..])
			.iter()
			.map(|result| match result {
				Ok(iota_vec) => Ok(StackState::new(iota_vec.clone(), stack_state.ravenmind.clone())),
				Err(action_error) => Err(*action_error),
			})
		))
	}
}

#[cfg(test)]
mod tests {
	use crate::{simulator::EntityType, hex_pattern::*};

	use super::*;

	#[test]
	fn entity_type_exclusive_commutativity() {
	static ENTITY_TYPES: [EntityType; 5] = [EntityType::Animal, EntityType::Monster, EntityType::Item, EntityType::Player, EntityType::Living];
		for self_type in ENTITY_TYPES.iter() {
			for other_type in ENTITY_TYPES.iter() {
				assert_eq!(self_type.mutually_exclusive(other_type), other_type.mutually_exclusive(self_type))
			}
		}
	}

	#[test]
	fn iota_display() {
		let pattern = Iota::Pattern(HexPattern { pattern_vec: vec![HexDir::W], start_dir: HexAbsoluteDir::East });
		let def_double: Iota = 124.1231.into();
		let indef_double: Iota = None.into();
		let def_vec: Iota = (123.0123, 63.0, -523.0).into();
		let indef_vec: Iota = Right(false).into();
		let widget = Iota::Widget;
		let entity = Iota::Entity(IotaEntity::new("Zombie")
			.add_guaranteed(EntityType::Monster)
			.add_guaranteed(EntityType::Living)
			.remove_possible(EntityType::Animal)
			.remove_possible(EntityType::Item)
			.remove_possible(EntityType::Player).into());
		let indef_list = Iota::List(IotaList::new(Right(Some(4))));
		let indef_list_unknown_length = Iota::List(IotaList::new(Right(None)));
		let def_list = Iota::List(IotaList::new(Left(vec![indef_list.clone(), indef_list_unknown_length.clone()])));

		assert_eq!(&pattern.to_string(), "HexPattern(W, East)");
		assert_eq!(&def_double.to_string(), "124.1231");
		assert_eq!(&indef_double.to_string(), "UNKOWN");
		assert_eq!(&def_vec.to_string(), "(123.0123, 63, -523)");
		assert_eq!(&indef_vec.to_string(), "(UNKNOWN, UNKNOWN, UNKNOWN ; guaranteed in range: false)");
		assert_eq!(&widget.to_string(), "Null");
		assert_eq!(&entity.to_string(), "Zombie");
		assert_eq!(&indef_list.to_string(), "[UNKOWN, len=4]");
		assert_eq!(&indef_list_unknown_length.to_string(), "[UNKOWN, len=UNKOWN]");
		assert_eq!(&def_list.to_string(), "[[UNKOWN, len=4], [UNKOWN, len=UNKOWN]]");
	}
}