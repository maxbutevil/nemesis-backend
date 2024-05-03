
use crate::Id;
use diesel::prelude::*;
use serde::{
	self,
	Serialize, Serializer,
	Deserialize, Deserializer
};

//use serde_json::Result;
//mod schema;

use crate::schema::*;

/*
#[derive(Debug)]
#[derive(Queryable, Selectable, Insertable)]
#[derive(Deserialize)]
#[diesel(table_name = schema::users)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct User {
	pub username: String,
	pub password: String,
}
*/

fn empty_string() -> String { "".to_string() }
/*
fn i32_to_bool(i: i32) -> bool { i != 0 }
fn bool_to_i32(b: bool) -> i32 { b as i32 }

fn serialize_bool<S: Serializer>(i: &i32, s: S) -> Result<S::Ok, S::Error> {
  s.serialize_bool(i32_to_bool(*i))
}
fn deserialize_bool<'de, D: Deserializer<'de>>(d: D) -> Result<i32, D::Error> {
	match bool::deserialize(d) {
		Ok(b) => Ok(bool_to_i32(b)),
		Err(err) => Err(err)
	}
}
*/


#[derive(Debug, Default)]
#[derive(Serialize, Deserialize)]
#[derive(Queryable, Selectable, Insertable)]
#[derive(AsChangeset)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct User {
	
	#[serde(default = "empty_string")]
	pub id: String,
	
	/* PRIVATE */
	#[serde(skip_serializing_if = "Option::is_none")]
	pub latitude: Option<f32>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub longitude: Option<f32>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub birth_date: Option<String>,
	
	/* VITALS */
	#[serde(skip_serializing_if = "Option::is_none")]
	pub name: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub gender_identity: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub pronouns: Option<String>,
	
	/* PROFILE */
	#[serde(skip_serializing_if = "Option::is_none")]
	pub bio: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub looking_for: Option<String>,
	
	#[serde(skip_serializing_if = "Option::is_none")]
	pub photos: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub interests: Option<String>,
	
}
impl User {
	
	pub fn new(id: Id) -> Self {
		Self { id: (*id).clone(), ..Default::default() }
	}
	
	pub fn location(&self) -> Option<(f32, f32)> {
		match (self.latitude, self.longitude) {
			(Some(latitude), Some(longitude)) => Some((latitude, longitude)),
			_ => None
		}
	}
	
	pub fn to_profile<'a>(self) -> Profile {
		
		Profile {
			
			id: self.id,
			
			/* DERIVED */
			age: None,
			
			/* VITALS */
			name: self.name,
			
			gender_identity: self.gender_identity,
			pronouns: self.pronouns,
			
			bio: self.bio,
			looking_for: self.looking_for,
			
			interests: self.interests,
			photos: self.photos
			
		}
		
	}
	
}

#[derive(Debug)]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
	
	#[serde(default = "empty_string")]
	pub id: String,
	
	/* DERIVED */
	// distance?
	#[serde(skip_serializing_if = "Option::is_none")]
	pub age: Option<usize>,
	
	/* VITALS */
	#[serde(skip_serializing_if = "Option::is_none")]
	pub name: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub gender_identity: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub pronouns: Option<String>,
	
	/* PROFILE */
	#[serde(skip_serializing_if = "Option::is_none")]
	pub bio: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub looking_for: Option<String>,
	
	#[serde(skip_serializing_if = "Option::is_none")]
	pub interests: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub photos: Option<String>,
	
}



#[derive(PartialEq)]
pub enum Sender {
	One,
	Two
}
pub enum MatchState {
	Dead,
	Pending(Sender),
	Active
}
impl Sender {
	
	pub fn get(sender_id: &Id, receiver_id: &Id) -> Self {
		
		if sender_id < receiver_id {
			Self::One
		} else {
			Self::Two
		}
		
	}
	pub fn to_i32(sender: &Self) -> i32 {
		match sender {
			Self::One => 0,
			Self::Two => 1
		}
	}
	pub fn from_i32(i: i32) -> Option<Self> {
		match i {
			0 => Some(Self::One),
			1 => Some(Self::Two),
			invalid => {
				println!("Invalid match state: {}", invalid);
				None
			}
		}
	}
	
}
impl MatchState {
	
	pub fn from_i32(i: i32) -> Option<Self> {
		match i {
			0 => Some(Self::Dead),
			1 => Some(Self::Active),
			2 => Some(Self::Pending(Sender::One)),
			3 => Some(Self::Pending(Sender::Two)),
			invalid => {
				println!("Invalid match state: {}", invalid);
				None
			}
		}
	}
	pub fn to_i32(state: &Self) -> i32 {
		match state {
			Self::Dead => 0,
			Self::Active => 1,
			Self::Pending(Sender::One) => 2,
			Self::Pending(Sender::Two) => 3,
		}
	}
}


#[derive(Debug)]
#[derive(Serialize, Deserialize)]
#[derive(Queryable, Selectable, Insertable)]
//#[derive(AsChangeset)]
#[diesel(table_name = matches)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Match {
	
	//#[serde(default = "empty_string")]
	pub user1: String,
	pub user2: String,
	
	pub state: i32,
	
}
impl Match {
	
	
	pub fn order<T>(one: T, two: T) -> (T, T)
		where T: PartialOrd<T>
	{
		if one <= two {
			(one, two)
		} else {
			(two, one)
		}
	}
	
	
	
	pub fn get_state(&self) -> Option<MatchState> {
		MatchState::from_i32(self.state)
	}
	pub fn set_state(&mut self, new_state: MatchState) {
		self.state = MatchState::to_i32(&new_state)
	}
	
	
	
	fn new_unchecked(user1: &Id, user2: &Id, state: MatchState) -> Self {
		Self {
			user1: (**user1).clone(),
			user2: (**user2).clone(),
			state: MatchState::to_i32(&state)
		}
	}
	pub fn new(user1: &Id, user2: &Id, state: MatchState) -> Self {
		let (user1, user2) = Self::order(user1, user2);
		Self::new_unchecked(user1, user2, state)
	}
	pub fn new_dead(user1: &Id, user2: &Id) -> Self {
		Self::new(user1, user2, MatchState::Dead)
	}
	pub fn new_liked(sender_id: &Id, receiver_id: &Id) -> Self {
		Self::new(
			sender_id, receiver_id,
			MatchState::Pending(Sender::get(sender_id, receiver_id))
		)
	}
	
}


#[derive(Debug)]
#[derive(Serialize, Deserialize)]
#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = messages)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ChatMessage {
	
	pub id: i32,
	
	pub user1: String,
	pub user2: String,
	pub sender: i32,
	
	pub time: String,
	pub content: String
	
}


/*
pub enum MatchState {
	Dead,
	OneLiked,
	TwoLiked,
	Active
}
impl Serialize for MatchState {
	
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		
		serializer.serialize_str(
			match self {
				MatchState::Dead => "dead",
				MatchState::OneLiked => "one_liked",
				MatchState::TwoLiked => "two_liked",
				MatchState::Active => "active"
			}
		)
		
	}
	
}
impl Deserialize for MatchState {
	
	fn deserialize<'de, D>(deserializer: D) -> Result<Self, D::Error>
		where D: Deserializer<'de>
	{
		
		match String::deserialize(deserializer) {
			Ok("dead") => Ok(MatchState::Dead),
			Ok("one_liked") => Ok(MatchState::OneLiked),
			Ok("two_liked") => Ok(MatchState::TwoLiked),
			Ok("active") => Ok(MatchState::Active),
			_ => Err(D::Error)
		}
		
	}
}
*/




/*
#[derive(Debug, Default)]
#[derive(Serialize, Deserialize)]
#[derive(Queryable, Selectable, Insertable)]
//#[derive(AsChangeset)]
#[diesel(table_name = impressions)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Impression {
	
	#[serde(default = "empty_string")]
	pub user_id: String,
	pub profile_id: String,
	
	#[serde(serialize_with = "serialize_bool")]
	#[serde(deserialize_with = "deserialize_bool")]
	pub liked: i32
	
}
*/



/*
impl Impression {
	
	
	pub fn new(user_id: Id, profile_id: Id, liked: bool) -> Impression {
		Self { user_id: (*user_id).clone(), profile_id: (*profile_id).clone(), liked: bool_to_i32(liked) }
	}
	/*pub fn new(user_id: String, profile_id: String, liked: bool) -> Impression {
		Self { user_id, profile_id, liked: bool_to_i32(liked) }
	}
	pub fn new(user_id: Id, profile_id: Id, liked: bool) {
		Self::new(*user_id, *profile_id, liked);
	}*/
	
	pub fn liked(&self) -> bool {
		i32_to_bool(self.liked)
	}
	
}
*/



/*
#[derive(Debug, Default)]
#[derive(Serialize, Deserialize)]
#[derive(Queryable, Selectable, Insertable)]
#[derive(AsChangeset)]
#[diesel(table_name = profiles)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Profile {
	
	#[serde(default = "empty_string")]
	pub id: String,
	
	#[serde(skip_serializing_if = "Option::is_none")]
	pub name: Option<String>,
	
	#[serde(skip_serializing_if = "Option::is_none")]
	pub gender_identity: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub pronouns: Option<String>,
	
	#[serde(skip_serializing_if = "Option::is_none")]
	pub bio: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub looking_for: Option<String>
	
}

impl Profile {
	
	pub fn new(id: String) -> Self {
		Self { id, ..Default::default() }
	}
	
}
*/
