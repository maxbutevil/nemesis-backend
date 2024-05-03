
use crate::Id;
use crate::schema;
use crate::models::{
	User,
	Match,
	ChatMessage,
	
	Profile,
	Sender,
	MatchState
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
	
	fn strip<T>(option: Option<T>) -> Option<()> {
		match option {
			Some(_) => Some(()),
			None => None
		}
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
	
	pub async fn read_user(&self, user_id: Id) -> Option<User> {
		
		use schema::users::{self, dsl::*};
		
		let id_clone = user_id.clone();
		let result = self.execute(move |connection| {
			users
				.select(User::as_select())
				.find(&*id_clone)
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
					.values(id.eq(&*id_clone))
					.execute(connection)
		).await;
		
		match result {
			Some(_) => Some(User::new(user_id)),
			None => None
		}
		
	}
	pub async fn write_user(&self, user: User) -> Option<()> {
		
		use schema::users;
		
		Self::strip(self.execute_expect(
			"Error on user write", 
			move |connection| {
				update(users::table.find(&user.id))
					.set(&user)
					.execute(connection)
		}).await)
		
	}
	
	pub async fn get_profile(&self, user_id: Id) -> Option<Profile> {
		
		use schema::users;
		
		let result: Option<User> = self.execute_expect(
			"Error getting profile",
			move |connection|
				users::table
					.select(User::as_select())
					.find(&*user_id)
					.first(connection)
		).await;
		
		match result {
			Some(user) => Some(user.to_profile()),
			None => None
		}
		
	}
	pub async fn get_profiles(&self, user_id: Id) -> Option<Vec<Profile>> {
		
		use schema::users::{self, dsl::*};
		use schema::matches::{self, dsl::*};
		
		let result: Option<Vec<User>> = self.execute_expect(
			"Error getting candidate profiles",
			move |connection| {
				
				let two_liked_one = MatchState::to_i32(&MatchState::Pending(Sender::Two));
				let one_liked_two = MatchState::to_i32(&MatchState::Pending(Sender::One));
				
				let ineligible_one = {
					matches::table
						.select(user2)
						.filter(user1.eq(&*user_id))
						.filter(state.ne(two_liked_one)) // allow unreciprocated likes from others
				};
				
				let ineligible_two = {
					matches::table
						.select(user1)
						.filter(user2.eq(&*user_id))
						.filter(state.ne(one_liked_two)) // allow unreciprocated likes from others
				};
				
				users::table
					.select(User::as_select())
					.filter(id.ne(&*user_id))
					.filter(id.ne_all(ineligible_one))
					.filter(id.ne_all(ineligible_two))
					.limit(5)
					.load::<User>(connection)
				
			}).await;
			
		match result {
			None => None,
			Some(candidate_users) => {
				Some(
					candidate_users
						.into_iter()
						.map(move |user| user.to_profile())
						.collect()
				)
			}
		}
		
	}
	
	pub async fn get_match_state(&self, id1: Id, id2: Id) -> Option<MatchState> {
		
		//let (id1, id2) = (id1.clone(), id2.clone());
		let (id1, id2) = Match::order(id1, id2);
		
		use schema::matches::{self, dsl::state};
		
		let result = self.execute(
			move |connection|
				matches::table
					.select(state)
					.find((&**id1, &**id2))
					.first::<i32>(connection)
		).await;
		
		match result {
			Some(i) => MatchState::from_i32(i),
			None => None
		}
		
	}
	pub async fn set_match_state(&self, id1: Id, id2: Id, new_state: MatchState) -> Option<()> {
		
		use schema::matches::{self, dsl::*};
		
		let (id1, id2) = Match::order(id1, id2);
		let new_state_i32 = MatchState::to_i32(&new_state);
		let new_match = Match::new(&id1, &id2, new_state);
		
		let result = self.execute_expect(
			"Error setting match state",
			move |connection|
				insert_into(matches::table)
					.values(&new_match)
					.on_conflict((user1, user2))
					.do_update()
					.set(state.eq(new_state_i32))
					.execute(connection)
		).await;
		
		match result {
			Some(_) => Some(()),
			None => None
		}
		
	}
	
	pub async fn put_message(&self, sender_id: Id, receiver_id: Id, content: String) -> Option<()> {
		
		use schema::messages::{self, dsl};
		
		let sender = Sender::get(&sender_id, &receiver_id);
		let (id1, id2) = Match::order(sender_id, receiver_id);
		
		let result = self.execute_expect(
			"Error inserting chat message",
			move |connection|
				insert_into(messages::table)
					.values((
						dsl::user1.eq(&**id1),
						dsl::user2.eq(&**id2),
						dsl::sender.eq(Sender::to_i32(&sender)),
						dsl::content.eq(&content)
					))
					.execute(connection)
		).await;
		
		match result {
			Some(_) => Some(()),
			None => None
		}
		
	}
	pub async fn get_chat_messages(&self, user1: Id, user2: Id, limit: i64, older_than: String) -> Option<Vec<ChatMessage>> {
		
		use schema::messages::{self, dsl};
		
		let (id1, id2) = Match::order(user1, user2);
		
		self.execute_expect(
			"Error getting chat history",
			move |connection|
				messages::table
					.select(ChatMessage::as_select())
					.filter(dsl::user1.eq(&*id1))
					.filter(dsl::user2.eq(&*id2))
					.filter(dsl::time.lt(older_than))
					.limit(limit)
					.load::<ChatMessage>(connection)
		).await
		
	}
	
}

