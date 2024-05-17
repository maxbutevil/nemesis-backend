
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
	fn users_to_profiles(users: Vec<User>) -> Vec<Profile> {
		
		users
			.into_iter()
			.map(move |user| user.to_profile())
			.collect()
		
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
	
	pub async fn get_user(&self, user_id: &Id) -> Option<User> {
		
		use schema::users;
		
		let user_id = user_id.clone();
		
		self.execute_expect(
			"Error getting user",
			move |connection|
				users::table
					.select(User::as_select())
					.find(&*user_id)
					.first::<User>(connection)
		).await
		
	}
	
	pub async fn handle_autolikes(&self, user_id: &Id) -> Option<()> {
		
		use schema::users;
		use schema::matches;
		
		let results: (Option<Vec<String>>, Option<Vec<String>>, Option<Vec<String>>) = tokio::join!(
			self.execute_expect(
				"Error getting autoliking users",
				move |connection| {
					users::table
						.select(users::id)
						.filter(users::id.like("autolike%"))
						.load::<String>(connection)
				}
			),
			self.execute_expect(
				"Error getting autoliking users",
				move |connection| {
					users::table
						.select(users::id)
						.filter(users::id.like("autodislike%"))
						.load::<String>(connection)
				}
			),
			self.execute_expect(
				"Error getting automatch users",
				move |connection| {
					users::table
						.select(users::id)
						.filter(users::id.like("automatch%"))
						.load::<String>(connection)
				}
			)
		);
		
		if let (Some(autolike_ids), Some(autodislike_ids), Some(automatch_ids)) = results {
			
			let user_id = user_id.clone(); // genuinely no clue why this is necessary, but it works
			Self::strip(
				self.execute_expect(
					"Error autoliking/disliking users",
					move |connection| {
						
						//let user_id = user_id.clone();
						//let user_id2 = user_id.clone();
						let likes = autolike_ids
							.into_iter()
							.map(|id| Id::new(id))
							.map(|id| Match::new(&id, &user_id,
								MatchState::Pending(Sender::of(&id, &user_id))));
						
						let dislikes = autodislike_ids
							.into_iter()
							.map(|id| Id::new(id))
							.map(|id| Match::new(&id, &user_id, MatchState::Dead));
						
						let matches = automatch_ids
							.into_iter()
							.map(|id| Id::new(id))
							.map(|id| Match::new(&id, &user_id, MatchState::Active));
						
						insert_into(matches::table)
							.values(
								likes
									.chain(dislikes)
									.chain(matches)
									.collect::<Vec<_>>()
							)
							.execute(connection)
							
					}
				).await
			)
			
		} else {
			None
		}
		/*
		let result = self.execute_expect();
		
		let user_id = user_id.clone();
		let result = self.execute_expect(
			"Error autoliking user",
			move |connection| {
				users::table
					.select(User::as_select())
					
					.find(&*user_id)
					.load::<User>(connection)
			}
		);
		*/
		
	}
	
	pub async fn read_user(&self, user_id: &Id) -> Option<User> {
		
		use schema::users::{self, dsl::*};
		
		let result = self.get_user(user_id).await;
		
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
			Some(_) => {
				self.handle_autolikes(&user_id).await;
				Some(User::new(user_id.clone()))
			},
			None => None
		}
		
	}
	pub async fn write_user(&self, user: User) -> Option<()> {
		
		use schema::users;
		
		Self::strip(
			self.execute_expect(
				"Error on user write", 
				move |connection| {
					update(users::table.find(&user.id))
						.set(&user)
						.execute(connection)
			}).await
		)
		
	}
	
	pub async fn get_profile(&self, user_id: &Id) -> Option<Profile> {
		
		self.get_user(&*user_id)
			.await
			.map(|user| user.to_profile())
		
	}
	pub async fn get_queue_profiles(&self, user_id: &Id, blacklist: Option<Vec<String>>) -> Option<Vec<Profile>> {
		
		use schema::users::{self, dsl::*};
		use schema::matches::{self, dsl::*};
		
		let user_id = user_id.clone();
		let result: Option<Vec<User>> = self.execute_expect(
			"Error getting candidate profiles",
			move |connection| {
				
				let two_liked_one = MatchState::Pending(Sender::Two);
				let one_liked_two = MatchState::Pending(Sender::One);
				
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
				
				let ineligible = ineligible_one.union(ineligible_two);
				
				// this one needs a strategy check
				users::table
					.select(User::as_select())
					.filter(id.ne(&*user_id))
					.filter(id.ne_all(ineligible))
					.filter(id.ne_all(blacklist.unwrap_or_default()))
					.limit(5)
					.load::<User>(connection)
				
			}).await;
		
		match result {
			None => None,
			Some(candidate_users) =>
				Some(Self::users_to_profiles(candidate_users))
		}
		
	}
	
	pub async fn get_match_state(&self, id1: &Id, id2: &Id) -> Option<MatchState> {
		
		//let (id1, id2) = (id1.clone(), id2.clone());
		let (id1, id2) = Match::order(id1.clone(), id2.clone());
		
		use schema::matches::{self, dsl::state};
		
		self.execute(
			move |connection|
				matches::table
					.select(state)
					.find((&**id1, &**id2))
					.first::<MatchState>(connection)
		).await
		
	}
	pub async fn set_match_state(&self, id1: &Id, id2: &Id, new_state: MatchState) -> Option<()> {
		
		use schema::matches::{self, dsl::*};
		
		let (id1, id2) = Match::order(id1, id2);
		let new_match = Match::new(id1, id2, new_state.clone());
		
		Self::strip(
			self.execute_expect(
				"Error setting match state",
				move |connection|
					insert_into(matches::table)
						.values(&new_match)
						.on_conflict((user1, user2))
						.do_update()
						.set(state.eq(&new_state))
						.execute(connection)
			).await
		)
		
	}
	
	pub async fn get_initial_chat_messages(&self, user_id: Id) -> Option<Vec<ChatMessage>> {
		
		use schema::{matches, messages};
		
		//let user_id_clone = user_id.clone();
		//let user_id = user_id.clone();
		self.execute_expect(
			"Error getting user recent messages",
			move |connection| {
				
				matches::table
					.filter(matches::state.eq(MatchState::Active))
					.filter(
						matches::user1.eq(&*user_id)
							.or(matches::user2.eq(&*user_id)))
					.inner_join(messages::table.on(
						matches::user1.eq(messages::user1)
							.and(matches::user2.eq(messages::user2))))
					.select(ChatMessage::as_select())
					.order(messages::timestamp.desc())
					// naively loads the last 20 messages - this will need to change
					.limit(20)
					//.order(messages)
					//.group()
					//.filter()
					.load::<ChatMessage>(connection)
				
				
			}
		).await	
		
	}
	pub async fn get_initial_match_profiles(&self, user_id: Id) -> Option<Vec<Profile>> {
		
		//use schema::users::{self, dsl::*};
		use schema::{users, matches};
		
		let result = self.execute_expect(
			"Error getting user matches",
			move |connection| {
				
				let users1 = matches::table
					.filter(matches::dsl::user1.eq(&*user_id))
					.filter(matches::state.eq(MatchState::Active))
					.inner_join(users::table.on(users::id.eq(matches::user2)))
					.select(User::as_select());
				
				let users2 = matches::table
					.filter(matches::dsl::user2.eq(&*user_id))
					.filter(matches::state.eq(MatchState::Active))
					.inner_join(users::table.on(users::id.eq(matches::user1)))
					.select(User::as_select());
				
				users1.union(users2).load::<User>(connection)
				
			}
				
		).await;
		
		result.map(|users| Self::users_to_profiles(users))
		
	}
	pub async fn put_chat_message(&self, sender_id: Id, receiver_id: Id, id: String, content: String) -> Option<()> {
		
		use schema::messages::{self, dsl};
		
		let sender = Sender::of(&sender_id, &receiver_id);
		let (id1, id2) = Match::order(sender_id, receiver_id);
		
		Self::strip(
			self.execute_expect(
				"Error inserting chat message",
				move |connection|
					insert_into(messages::table)
						.values((
							dsl::id.eq(&id),
							dsl::user1.eq(&**id1),
							dsl::user2.eq(&**id2),
							dsl::sender.eq(&sender),
							dsl::content.eq(&content)
						))
						.execute(connection)
			).await
		)
		
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
					.filter(dsl::timestamp.lt(older_than))
					.limit(limit)
					.load::<ChatMessage>(connection)
		).await
		
	}
	
}