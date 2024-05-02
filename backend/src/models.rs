
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

fn serialize_bool<S: Serializer>(i: &i32, s: S) -> Result<S::Ok, S::Error> {
  s.serialize_bool(*i != 0)
}
fn deserialize_bool<'de, D: Deserializer<'de>>(d: D) -> Result<i32, D::Error> {
	match bool::deserialize(d) {
		Ok(i) => Ok(i as i32),
		Err(err) => Err(err)
	}
}



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

/*
#[derive(Debug)]
#[derive(Serialize)]
pub struct Profile<'a> {
	
	#[serde(default = "empty_string")]
	pub id: &'a str,
	
	/* DERIVED */
	// distance?
	#[serde(skip_serializing_if = "Option::is_none")]
	pub age: Option<usize>,
	
	/* VITALS */
	#[serde(skip_serializing_if = "Option::is_none")]
	pub name: &'a Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub gender_identity: &'a Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub pronouns: &'a Option<String>,
	
	/* PROFILE */
	#[serde(skip_serializing_if = "Option::is_none")]
	pub bio: &'a Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub looking_for: &'a Option<String>,
	
	#[serde(skip_serializing_if = "Option::is_none")]
	pub interests: &'a Option<String>,
	
	#[serde(skip_serializing_if = "Option::is_none")]
	pub photos: &'a Option<String>,
	
}*/

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



impl User {
	
	pub fn new(id: String) -> Self {
		Self { id, ..Default::default() }
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
	
	
	/*
	pub fn as_profile<'a>(&'a self) -> Profile<'a> {
		
		Profile {
			
			id: &self.id,
			
			/* DERIVED */
			age: None,
			
			/* VITALS */
			name: &self.name,
			
			gender_identity: &self.gender_identity,
			pronouns: &self.pronouns,
			
			bio: &self.bio,
			looking_for: &self.looking_for,
			
			interests: &self.interests,
			photos: &self.photos
			
		}
		
	}
	*/
	
}





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
