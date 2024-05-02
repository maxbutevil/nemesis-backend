

use crate::schema;
use crate::models::{
	User,
	Profile
};

use diesel::prelude::*;
use diesel::{insert_into, update};
use deadpool_diesel::sqlite::{Runtime, Manager, Pool};

use std::fmt::Display;

#[derive(Clone)]
pub struct DatabaseState {
	connections: Pool
}


impl DatabaseState {
	
	fn get_url() -> String {
	
		use dotenvy::dotenv;
		use std::env;
		
		dotenv().expect(".env file not found");
		env::var("DATABASE_URL").expect("DATABASE_URL must be set")
		
	}
	
	pub fn new() -> Self {
		
		let manager = Manager::new(Self::get_url(), Runtime::Tokio1);
		let connections = Pool::builder(manager)
			.max_size(8)
			.build()
			.expect("Error creating Sqlite connection pool");
		
		Self { connections }
		
	}
	
	async fn execute_result<T, E, F>(&self, query: F) -> Option<Result<T, E>>
	where
		F: Send + 'static + FnOnce(&mut SqliteConnection) -> Result<T, E>,
		T: Send + 'static,
		E: Send + 'static
	{
		
		let connection = self.connections.get().await;
		
		match connection {
			Err(err) => println!("{err}"),
			Ok(connection) => {
				match connection.interact(query).await {
					Err(err) => println!("{err}"),
					Ok(output) => return Some(output)
				}
			}
		};
		
		None
		
	}
	
	async fn execute_expect<T, F, E>(&self, message: &'static str, query: F) -> Option<T>
	where
		F: Send + 'static + FnOnce(&mut SqliteConnection) -> Result<T, E>,
		T: Send + 'static,
		E: Send + 'static + Display
	{
		
		match self.execute_result(query).await {
			Some(Ok(output)) => Some(output),
			Some(Err(err)) => {
				println!("{message}: {err}");
				None
			},
			_ => None
		}
		
	}
	async fn execute<T, E, F>(&self, query: F) -> Option<T>
	where
		F: Send + 'static + FnOnce(&mut SqliteConnection) -> Result<T, E>,
		T: Send + 'static,
		E: Send + 'static
	{
		
		match self.execute_result(query).await {
			Some(Ok(output)) => Some(output),
			_ => None
		}
		
	}
	
	
	pub async fn write_user(&self, user: User) -> Option<()> {
		
		use schema::users::dsl::*;
		
		let result = self.execute_expect(
			"Error on user write", 
			move |connection| {
				update(users.find(&user.id))
					.set(&user)
					.execute(connection)
		}).await;
		
		match result {
			Some(_) => Some(()),
			None => None
		}
		
		/*
		match result {
			Some(Ok(_)) => Ok(()),
			Some(Err(err)) => {
				println!("Error on user write: {err}");
				Err(())
			}
			None => Err(())
		}
		*/
		
	}
	
	
	
	/*
	pub async fn select_user(&self, user_id: String) -> Option<Result<User, diesel::result::Error>> {
		
		use schema::users::dsl::*;
		
		self.execute(move |connection| {
			users.find(user_id)
				.select(User::as_select())
				.first(connection)
		}).await
		
	}
	pub async fn get_user(&self, user_id: String) -> Option<User> {
		Self::flatten(self.select_user(user_id).await)
	}
	pub async fn get_user_loud(&self, user_id: String, message: &'static str) -> Option<User> {
		Self::flatten_loud(self.select_user(user_id).await, message)
	}
	*/
	
	pub async fn read_user(&self, user_id: String) -> Option<User> {
		
		use schema::users::{self, dsl::*};
		
		let id_clone = user_id.clone();
		let result = self.execute(move |connection| {
			users.find(id_clone)
				.select(User::as_select())
				.first(connection)
		}).await;
		
		if let Some(user) = result {
			return Some(user);
		}
		
		// User doesn't exist, let's create it
		let id_clone = user_id.clone();
		let result = self.execute_expect(
			"Error inserting user on read",
			move |connection|
				insert_into(users::table)
					.values(id.eq(id_clone))
					.execute(connection)
		).await;
		
		match result {
			Some(_) => Some(User::new(user_id)),
			None => None
		}
		
	}
	
	pub async fn get_profiles(&self, _user_id: String) -> Option<Vec<Profile>> {
		
		//let user_result = self.get_user_(user_id.clone());
		
		//use schema::users::self;
		
		let result = self.execute_expect(
			"Error getting matches",
			|connection| {
				
				use schema::users;
				
				users::table
					//.filter(id.ne(user_id))
					.limit(5)
					.select(User::as_select())
					.load::<User>(connection)
			}).await;
			
		match result {
			None => None,
			Some(users) => {
				Some(
					users
						.into_iter()
						.map(move |user| user.to_profile())
						.collect()
				)
			}
		}
		
	}
	
	/*
	pub async fn write_profile(&self, profile: Profile) -> Result<(), ()> {
		
		use schema::profiles::{self};//, dsl::*};
		
		let result = self.execute(move |connection| {
			/*
			insert_into(profiles::table)
				.values(&profile)
				.on_conflict(id)
				.do_update()
				.set(&profile)
				.execute(connection)
			*/
			update(profiles::table.find(&profile.id))
				.set(&profile)
				.execute(connection)
		}).await;
		
		match result {
			Some(Ok(_)) => return Ok(()),
			Some(Err(err)) => println!("{err}"),
			_ => {}
		}
		
		Err(())
		
	}
	
	pub async fn read_profile(&self, user_id: String) -> Option<Profile> {
		
		use schema::profiles::{self, dsl::*};
		
		let id_clone = user_id.clone();
		let result = self.execute(move |connection| {
			profiles
				.find(id_clone)
				.select(Profile::as_select())
				.first(connection)
		}).await;
		
		match result {
			Some(Ok(profile)) => return Some(profile),
			Some(Err(err)) => println!("{err}"),
			_ => {}
		}
		
		// If profile doesn't exist, let's try to create it
		let id_clone = user_id.clone();
		let result = self.execute(move |connection| {
			insert_into(profiles::table)
				.values(id.eq(id_clone))
				.execute(connection)
		}).await;
		
		match result {
			Some(Ok(count)) => {
				if count > 0 {
					return Some(Profile::new(user_id));
				} else {
					println!("Error on profile read insert")
				}
			},
			Some(Err(err)) => println!("{err}"),
			_ => {}
		};
		
		None
		
	}
	*/
	
}

