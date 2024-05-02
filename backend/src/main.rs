


pub mod schema;
pub mod models;
pub mod db;
pub mod ws;
pub mod id;
pub use id::Id;

use models::*;
use db::DatabaseState;
use ws::{
	WebSocketState,
	WebSocketUpgrade,
	Message
};

// look into tokio tracing
//use std::sync::Arc;

//use db::models::*;

//use axum::Error;
use axum::http::StatusCode;
use axum::response::Response; // for websocket upgrade
use axum::extract::{Request, State, FromRef, Json};

use firebase_auth::{
	FirebaseAuth,
	FirebaseAuthState,
	FirebaseUser
};



/*
use axum::{
	Error,
	extract::ws::{WebSocket, Message}};
*/
//use futures_util::{sink::SinkExt, stream::{StreamExt, SplitSink, SplitStream}};

//type WebSocketState = ;
use std::sync::Arc;



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

#[tokio::main]
async fn main() {
	
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
		//.route("/profiles", get(get_profiles))
		.fallback(not_found)
		.with_state(state);
	
	let listener = tokio::net::TcpListener::bind("0.0.0.0:5050").await.unwrap();
	println!("Listening!");
	axum::serve(listener, router).await.expect("Axum server error");
	
}

async fn not_found(request: Request) {
	println!("Invalid endpoint: {}", request.uri());
}

async fn get_profiles(State(db): State<DatabaseState>, auth: FirebaseUser) -> Result<(StatusCode, Json<Vec<Profile>>), StatusCode> {
	
	let id = auth.user_id;
	let result = db.get_profiles(id.clone()).await;
	
	match result {
		
		None => {
			println!("Error getting user matches [{}]", id);
			Err(StatusCode::UNAUTHORIZED)
		},
		Some(matches) => {
			println!("Getting user matches [{}]", id);
			Ok((StatusCode::OK, Json(matches)))
		}
		
	}
	
}

async fn read_user(State(db): State<DatabaseState>, auth: FirebaseUser) -> Result<(StatusCode, Json<User>), StatusCode> {
	
	let id = auth.user_id;
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
	
	println!("{:?}", user);
	
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




async fn ws_upgrade(request: WebSocketUpgrade, State(ws): State<WebSocketState>, auth: FirebaseUser) -> Response {
	request.on_upgrade(move |socket|
		ws.handle_socket(Id::new(auth.user_id), socket, on_message))
}
async fn on_message(ws: WebSocketState, id: Id, _message: Message) {
	
	ws.send(&id, Message::Text("wow".to_string())).await;
	
}

/*
async fn handle_rec
*/

