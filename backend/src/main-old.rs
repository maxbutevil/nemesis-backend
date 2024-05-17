


pub mod schema;
pub mod models;
pub mod db;
pub mod ws;
pub mod id;
pub mod http;
pub use id::Id;

use models::*;
use db::DatabaseState;

use http::InitialMatchData;
use ws::{
	WebSocket,
	WebSocketState,
	WebSocketUpgrade,
	IncomingMessage,
	OutgoingMessage
};


// look into tokio tracing
//use std::sync::Arc;

//use db::models::*;

//use axum::Error;
use std::sync::Arc;
use axum::http::StatusCode;
use axum::response::Response; // for websocket upgrade
use axum::extract::{Request, State, FromRef, Json};

use firebase_auth::{
	FirebaseAuth,
	FirebaseAuthState,
	FirebaseUser
};

use uuid::Uuid;

/*
use axum::{
	Error,
	extract::ws::{WebSocket, Message}};
*/
//use futures_util::{sink::SinkExt, stream::{StreamExt, SplitSink, SplitStream}};

//type WebSocketState = ;




#[derive(Clone)]
struct AppState {
	db: DatabaseState,
	auth: FirebaseAuthState,
	ws: WebSocketState
}

impl AppState {
	
	async fn new() -> Self {
		
		let db = DatabaseState::new();
		let ws = WebSocketState::new();
		
		let auth = FirebaseAuthState {
			firebase_auth: Arc::new(
				FirebaseAuth::new("nemesis-finder").await
			)
		};
		
		Self { db, ws, auth }
		
	}
	
}
impl FromRef<AppState> for DatabaseState {
	fn from_ref(app_state: &AppState) -> DatabaseState {
		app_state.db.clone()
	}
}
impl FromRef<AppState> for FirebaseAuthState {
	fn from_ref(app_state: &AppState) -> FirebaseAuthState {
		app_state.auth.clone()
	}
}
impl FromRef<AppState> for WebSocketState {
	fn from_ref(app_state: &AppState) -> WebSocketState {
		app_state.ws.clone()
	}
}


/*
impl FromRef<AppState> for ClientsState {
	
}
*/

/* look into direct UUID encoding via serde? */

#[tokio::main]
async fn main() {
	
	/*
	println!("{:?}", serde_json::to_string::<OutgoingMessage>(
		&OutgoingMessage::Match { user_id: "hello!".to_string() }
		//&IncomingMessage::Impression { user_id: "hello".to_string(), liked: true }
	));
	*/
	
	//println!("{:?}", serde_json::from_str::<Profile>("{\"name\": \"hi\"}"));
	//println!("{:?}", serde_json::from_str::<User>("{\"latitude\":38.9170983,\"longitude\":-119.9272283,\"birthDate\":\"null\"}"));
	
	let state = AppState::new().await;
	
	use axum::Router;
	use axum::routing::{get, post};
	
	let self_router = Router::new()
		.route("/read", get(read_user))
		.route("/write", post(write_user));
	
	let router = Router::new()
		.nest("/self", self_router)
		.route("/ws", get(ws_upgrade))
		.route("/discover", get(get_discover))
		.route("/matches", get(get_match_data))
		.fallback(not_found)
		.with_state(state);
	
	let listener = tokio::net::TcpListener::bind("0.0.0.0:5050").await.unwrap();
	println!("Listening!");
	axum::serve(listener, router).await.expect("Axum server error");
	
}

async fn not_found(request: Request) {
	println!("Invalid endpoint: {}", request.uri());
}

async fn get_discover(State(db): State<DatabaseState>, auth: FirebaseUser)
	-> Result<(StatusCode, Json<Vec<Profile>>), StatusCode> {
	
	let id = Id::new(auth.user_id);
	let result = db.get_discovery_profiles(id.clone()).await;
	
	match result {
		
		None => {
			println!("Error getting user discovery candidates [{}]", id);
			Err(StatusCode::UNAUTHORIZED)
		},
		Some(profiles) => {
			println!("Getting user discovery candidates [{}]", id);
			Ok((StatusCode::OK, Json(profiles)))
		}
		
	}
	
}
async fn get_match_data(State(db): State<DatabaseState>, auth: FirebaseUser)
	-> Result<(StatusCode, Json<InitialMatchData>), StatusCode> {
	
	let id = Id::new(auth.user_id);
	
	let result = tokio::join!(
		db.get_matches(id.clone()),
		db.get_initial_chat_messages(id.clone())
	);
	
	match result {
		(Some(matches), Some(messages)) => {
			println!("Getting user matches [{}]", id);
			Ok((StatusCode::OK, Json(
				InitialMatchData::new(matches, messages, &id))))
		},
		(Some(matches), None) => {
			println!("Error getting user messages (matches OK) [{}]", id);
			Ok((StatusCode::OK, Json(
				InitialMatchData::new(matches, Vec::new(), &id))))
		},
		_ => {
			println!("Error getting user matches [{}]", id);
			Err(StatusCode::UNAUTHORIZED)
		}
	}
	
}

async fn read_user(State(db): State<DatabaseState>, auth: FirebaseUser) -> Result<(StatusCode, Json<User>), StatusCode> {
	
	let id = Id::new(auth.user_id);
	let result = db.read_user(id.clone()).await;
	
	match result {
		None => {
			println!("Error reading user [{}]", id);
			Err(StatusCode::NOT_FOUND)
		},
		Some(user) => {
			println!("Reading user [{}]", id);
			Ok((StatusCode::OK, Json(user)))
		}
	}
	
}
async fn write_user(State(db): State<DatabaseState>, auth: FirebaseUser, Json(mut user): Json<User>) -> StatusCode {
	
	//println!("{:?}", user);
	
	// mild hack
	let id = auth.user_id;
	user.id = id.clone();
	
	let result = db.write_user(user).await;
	
	match result {
		None => {
			println!("Error writing to user [{}]: ", id);
			StatusCode::UNAUTHORIZED
		}
		Some(_) => {
			println!("Wrote to user [{}]", id);
			StatusCode::OK
		},
	}
	
}


async fn ws_upgrade(State(db): State<DatabaseState>, State(ws): State<WebSocketState>, auth: FirebaseUser, request: WebSocketUpgrade) -> Response {
	
	let id = Id::new(auth.user_id);
	println!("WebSocket upgrade [{}]", id);
	
	request.on_upgrade(move |socket|
		handle_socket(db, ws, id, socket))
	
}

async fn handle_socket(db: DatabaseState, ws: WebSocketState, id: Id, socket: WebSocket) {
	
	use ws::Message;
	use futures_util::StreamExt;
	
	let mut receiver = ws.listen(&id, socket);
	
	while let Some(message) = receiver.next().await {
		
		match message {
			Ok(Message::Text(data)) => {
				
				let result = serde_json::from_str::<IncomingMessage>(&data);
					
				match result {
					Err(err) => println!("IncomingMessage deserialization error: {}", err),
					Ok(message) => handle_message(&db, &ws, &id, message).await
				}
				
			},
			Ok(Message::Close(_)) => {},
			Ok(message) => println!("Invalid message received: {:?}", message),
			Err(err) => println!("WebSocket receiver error: {}", err)
		}
		
	}
	
	ws.drop(&id).await;
	
}
async fn handle_message(db: &DatabaseState, ws: &WebSocketState, sender_id: &Id, message: IncomingMessage) {
	
	match message {
		
		IncomingMessage::Impression { receiver_id, liked } => {
			
			let (db, ws, sender_id) = (db.clone(), ws.clone(), sender_id.clone());
			
			tokio::spawn(async move {
				handle_impression(db, ws, sender_id, Id::new(receiver_id), liked).await
			});
			
		},
		IncomingMessage::ChatMessage { receiver_id, content } => {
			
			let (db, ws, sender_id) = (db.clone(), ws.clone(), sender_id.clone());
			
			tokio::spawn(async move {
				handle_chat_message(db, ws, sender_id, Id::new(receiver_id), content).await
			});
			
		}
		
	}
	
}
async fn handle_impression(db: DatabaseState, ws: WebSocketState, sender_id: Id, receiver_id: Id, liked: bool) {
	
	//println!("Handling impression: {sender_id} -> {receiver_id} | {liked}");
	
	if !liked {
		db.set_match_state(sender_id, receiver_id, MatchState::Dead).await;
	} else {
		
		let current_state = db.get_match_state(sender_id.clone(), receiver_id.clone()).await;
		
		match current_state {
			None => handle_pending_like(db, ws, sender_id, receiver_id).await,
			Some(MatchState::Pending(old_sender)) => {
				
				// Ensure that both liked each other, rather than one being duplicated
				let new_sender = Sender::of(&sender_id, &receiver_id);
				if new_sender != old_sender {
					handle_match(db, ws, sender_id, receiver_id).await;
				} else {
					// Maybe log like duplication?
				}
				
			},
			_ => {} // dead/active matches ignore new likes. Log?
		}
		
	}
	
}
async fn handle_pending_like(db: DatabaseState, ws: WebSocketState, sender_id: Id, receiver_id: Id) {
	
	println!("New pending like: [{}] -> [{}]", sender_id, receiver_id);
	let new_state = MatchState::Pending(Sender::of(&sender_id, &receiver_id));
	
	tokio::join!(
		db.set_match_state(sender_id, receiver_id.clone(), new_state),
		ws.try_send(&receiver_id, OutgoingMessage::Like)
	);
	
}
async fn handle_match(db: DatabaseState, ws: WebSocketState, sender_id: Id, receiver_id: Id) {
	
	let profiles = tokio::join!(
		db.get_profile(sender_id.clone()),
		db.get_profile(receiver_id.clone())
	);
	
	match profiles {
		
		(Some(sender), Some(receiver)) => {
			println!("New match [{}] <-> [{}]", sender_id, receiver_id);
			tokio::join!(
				db.set_match_state(sender_id.clone(), receiver_id.clone(), MatchState::Active),
				ws.try_send(&sender_id, OutgoingMessage::Match { profile: receiver }),
				ws.try_send(&receiver_id, OutgoingMessage::Match { profile: sender })
			);
		},
		(Some(_), None) => println!("Match Error: Couldn't get receiver [{}]", receiver_id),
		(None, Some(_)) => println!("Match Error: Couldn't get sender [{}]", sender_id),
		(None, None) => println!("Match Error: Couldn't get profiles [{}] [{}]", sender_id, receiver_id)
		
	}
	
	
	
}
async fn handle_chat_message(db: DatabaseState, ws: WebSocketState, sender_id: Id, receiver_id: Id, content: String) {
	
	let message_id = Uuid::new_v4().to_string();
	
	tokio::join!(
		// unfortunate clone of content and id here
		ws.try_send(&receiver_id, OutgoingMessage::ChatMessage {
			sender_id: sender_id.to_string(),
			message_id: message_id.clone(),
			content: content.clone()
		}),
		db.put_chat_message(sender_id, receiver_id.clone(), message_id, content)
	);
	
}





