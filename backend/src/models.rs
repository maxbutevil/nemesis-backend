
use crate::Id;
use crate::schema::*;

use diesel::prelude::*;
use diesel::backend::Backend;
use diesel::expression::AsExpression;
use diesel::sql_types::Integer;
use diesel::deserialize::{self,	FromSql, FromSqlRow};
use diesel::serialize::{self,	ToSql, Output};

use serde::{Serialize, Deserialize};



fn empty_string() -> String { "".to_string() }


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
	
	fn distance_to_user(&self, other: &User) -> Option<usize> {
		// This is not even slightly accurate
		// Aggressively ballparking until I get around to improving it
		match (self.latitude, self.longitude, other.latitude, other.longitude) {
			(Some(lat1), Some(long1), Some(lat2), Some(long2)) => {
				let dlat = lat1 - lat2;
				let dlong = long1 - long2;
				Some((f32::sqrt(dlat * dlat + dlong * dlong)/60.0) as usize)
			}
			_ => None
		}
		
	}
	pub fn to_profile(self/* , for_user: &User */) -> Profile {
		
		//let distance = self.distance_to_user(for_user);
		
		Profile {
			
			id: self.id,
			
			/* DERIVED */
			// Calculating these in a secure way is honestly very annoying
			distance: None,
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
	pub distance: Option<usize>,
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



#[derive(Debug, Clone, PartialEq)]
#[derive(AsExpression, FromSqlRow)]
#[diesel(sql_type = Integer)]
pub enum Sender {
	One,
	Two
}
impl Sender {
	
	pub fn of(sender_id: &Id, receiver_id: &Id) -> Self {
		
		if sender_id < receiver_id {
			Self::One
		} else {
			Self::Two
		}
		
	}
	
	pub fn other(&self) -> Sender {
		match self {
			Sender::One => Sender::Two,
			Sender::Two => Sender::One
		}
	}
	
}
impl<DB> ToSql<Integer, DB> for Sender
	where DB: Backend, i32: ToSql<Integer, DB> {
  fn to_sql<'a>(&'a self, out: &mut Output<'a, '_, DB>) -> serialize::Result {
    match self {
			Sender::One => 0.to_sql(out),
			Sender::Two => 1.to_sql(out)
		}
  }
}
impl<DB> FromSql<Integer, DB> for Sender
	where DB: Backend, i32: FromSql<Integer, DB> {
	
	fn from_sql(bytes: DB::RawValue<'_>) -> deserialize::Result<Self> {
		match i32::from_sql(bytes)? {
			0 => Ok(Sender::One),
			1 => Ok(Sender::Two),
			other => Err(format!("Invalid Sender variant: {other}").into())
		}
	}
	
}



#[derive(Debug, Clone)]
#[derive(AsExpression, FromSqlRow)]
#[diesel(sql_type = Integer)]
pub enum MatchState {
	Dead,
	Pending(Sender),
	Active
}
impl<DB: Backend> ToSql<Integer, DB> for MatchState
	where i32: ToSql<Integer, DB> {
  fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, DB>) -> serialize::Result {
    match *self {
			MatchState::Dead => 0.to_sql(out),
			MatchState::Active => 1.to_sql(out),
			MatchState::Pending(Sender::One) => 2.to_sql(out),
			MatchState::Pending(Sender::Two) => 3.to_sql(out)
		}
  }
}
impl<DB: Backend> FromSql<Integer, DB> for MatchState 
	where i32: FromSql<Integer, DB> {
	fn from_sql(bytes: DB::RawValue<'_>) -> deserialize::Result<Self> {
		match i32::from_sql(bytes)? {
			0 => Ok(MatchState::Dead),
			1 => Ok(MatchState::Active),
			2 => Ok(MatchState::Pending(Sender::One)),
			3 => Ok(MatchState::Pending(Sender::Two)),
			other => Err(format!("Invalid MatchState variant: {other}").into())
		}
	}
	
}




/*
impl Into<i32> for MatchState {
	
	fn into(self) -> i32 {
		match self {
			MatchState::Dead => 0,
			MatchState::Active => 1,
			MatchState::Pending(Sender::One) => 2,
			MatchState::Pending(Sender::Two) => 3
		}
	}
	
}
impl Into<MatchState> for i32 {
	
	fn into(self) -> MatchState {
		
		match self {
			0 => MatchState::Dead,
			1 => MatchState::Active,
			2 => MatchState::Pending(Sender::One),
			3 => MatchState::Pending(Sender::Two),
			invalid => {
				println!("Invalid match state: {}", invalid);
				MatchState::Dead
			}
		}
		
	}
	
}
*/

/*
impl Serialize for MatchState {
	
}
impl Deserialize for MatchState {
	
}
*/



#[derive(Debug)]
//#[derive(Serialize, Deserialize)]
#[derive(Queryable, Selectable, Insertable)]
//#[derive(AsChangeset)]
#[diesel(table_name = matches)]
//#[diesel(belongs_to(User))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
//#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Match {
	
	//#[serde(default = "empty_string")]
	pub user1: String,
	pub user2: String,
	
	//pub state: i32,
	pub state: MatchState
	
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
	
	fn new_unchecked(user1: &Id, user2: &Id, state: MatchState) -> Self {
		Self {
			user1: user1.to_string(),
			user2: user2.to_string(),
			state //MatchState::to_i32(&state)
		}
	}
	pub fn new(user1: &Id, user2: &Id, state: MatchState) -> Self {
		let (user1, user2) = Self::order(user1, user2);
		Self::new_unchecked(user1, user2, state)
	}
	/*pub fn new_dead(user1: &Id, user2: &Id) -> Self {
		Self::new(user1, user2, MatchState::Dead)
	}
	pub fn new_liked(sender_id: &Id, receiver_id: &Id) -> Self {
		Self::new(
			sender_id, receiver_id,
			MatchState::Pending(Sender::of(sender_id, receiver_id))
		)
	}*/
	
}


#[derive(Debug)]
//#[derive(Serialize, Deserialize)]
#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = messages)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
//#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ChatMessage {
	
	pub id: String,
	
	pub user1: String,
	pub user2: String,
	pub sender: Sender,
	
	pub timestamp: String,
	pub content: String
	
}

impl ChatMessage {
	
	pub fn other_user(&self, user_id: &Id) -> &String {
		
		let user_id = user_id.to_string();
		
		if user_id == self.user1 {
			&self.user2
		} else if user_id == self.user2 {
			&self.user1
		} else {
			println!("ChatMessage other_user error; neither user present: {} | {:?}", user_id, self);
			&self.user1
		}
		
	}
	
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
