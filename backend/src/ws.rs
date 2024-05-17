


use crate::Id;

use crate::models::Profile;

pub use axum::extract::ws::{
	WebSocketUpgrade,
	WebSocket,
	Message
};



use std::sync::Arc;
use dashmap::DashMap;

//use std::future::Future;

use serde::{Serialize, Deserialize};

/*//type Req<In, Out>;
#[derive(Debug)]
#[derive(Deserialize)]
pub enum Request {
	
}*/

/*
struct ReqRes {
	
}
*/

#[derive(Debug)]
#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum IncomingMessage {
	
	QueueRefresh { blacklist: Option<Vec<String>> },
	
	Impression { to_id: String, liked: bool },
	ChatMessage { to_id: String, content: String }
	
}

#[derive(Debug)]
#[derive(Serialize)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum OutgoingMessage {
	
	QueueRefresh { profiles: Vec<Profile> },
	
	Like,
	Match { #[serde(flatten)] profile: Profile },
	ChatMessage { from_id: String, message_id: String, content: String }
}


/*#[derive(Debug)]
#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum IncomingRequest {
	
	Discovery { blacklist: Vec<String> }
	
}

#[derive(Debug)]
#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum IncomingEvent {
	
	Impression { to_id: String, liked: bool },
	ChatMessage { to_id: String, content: String }
	
}

#[derive(Debug)]
#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum Incoming {
	
	Request(IncomingRequest),
	Event(IncomingEvent)
	
}


#[derive(Debug)]
#[derive(Serialize)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum OutgoingResponse {
	Like,
	Match { #[serde(flatten)] profile: Profile },
	ChatMessage { from_id: String, message_id: String, content: String }
}

#[derive(Debug)]
#[derive(Serialize)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum OutgoingEvent {
	Like,
	Match { #[serde(flatten)] profile: Profile },
	ChatMessage { from_id: String, message_id: String, content: String }
}

#[derive(Debug)]
#[derive(Serialize)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum Outgoing {
	Response(OutgoingResponse),
	Event(OutgoingEvent)
}*/

use futures_util::{
	sink::SinkExt,
	stream::{SplitSink, SplitStream, StreamExt}
};
pub type WebSocketReceiver = SplitStream<WebSocket>;
pub type WebSocketSender = SplitSink<WebSocket, Message>;

#[derive(Clone)]
pub struct WebSocketState {
	clients: Arc<DashMap<Id, Client>>
}


struct Client {
	sender: WebSocketSender
}

impl Client {
	
	async fn send(&mut self, message: Message) -> Result<(), ()> {
		
		let result = self.sender.send(message).await;
		
		match result {
			Ok(_) => Ok(()),
			Err(err) => {
				dbg!(err);
				//sender.close().await;
				Err(())
			}
		}
		
	}
	/*
	fn send_all(&self, message: impl Iterator<Message>) {
		
		let result = self.sender.send_all()
		
	}
	*/
	
}


impl WebSocketState {
	
	pub fn new() -> Self {
		Self {
			clients: Arc::new(DashMap::new()),
		}
	}
	
	pub async fn to_incoming(message: Result<Message, axum::Error>) -> Option<IncomingMessage> {
		
		match message {
			Ok(Message::Text(data)) => {
				
				let result = serde_json::from_str(&data);
					
				match result {
					Err(err) => println!("IncomingMessage deserialization error: {}", err),
					Ok(message) => return Some(message)
				}
				
			},
			Ok(Message::Close(_)) => {},
			Ok(message) => println!("Invalid message received: {:?}", message),
			Err(err) => println!("WebSocket receiver error: {}", err)
		}
		
		None
		
	}
	
	pub async fn listen<F>(&self, id: Id, socket: WebSocket, on_message: F)
		where F: Fn(IncomingMessage)
	{
		
		let (sender, mut receiver) = socket.split();
		let _old_client = self.clients.insert(id.clone(), Client { sender });
		
		/*if let Some(old) = old {
			// Do something? Does this matter?
		}*/
		
		while let Some(message) = receiver.next().await {
			
			match message {
				Ok(Message::Text(data)) => {
					
					let result = serde_json::from_str::<IncomingMessage>(&data);
						
					match result {
						Err(err) => println!("IncomingMessage deserialization error: {}", err),
						Ok(message) => on_message(message)
					}
					
				},
				Ok(Message::Close(_)) => {},
				Ok(message) => println!("Invalid message received: {:?}", message),
				Err(err) => println!("WebSocket receiver error: {}", err)
			}
			
		}
		
		self.drop_client(&id).await
		
	}
	
	/*
	pub fn add_client(&self, id: &Id, socket: WebSocket) -> WebSocketReceiver {
		
		
		let (sender, receiver) = socket.split();
		
		
		
		/*
		if let Some(old) = old {
			// Do something? Does this matter?
		}
		*/
		
		receiver
		
	}*/
	pub async fn drop_client(&self, id: &Id) {
		self.clients.remove(id);
	}
	
	pub async fn has_id(&self, id: &Id) -> bool {
		self.clients.contains_key(id)
	}
	
	pub async fn try_send(&self, id: &Id, message: OutgoingMessage) -> Option<Result<(), ()>> {
		
		match self.clients.get_mut(id) {
			None => None,
			Some(mut client) => {
				
				let result = serde_json::to_string(&message);
			
				match result {
					Err(err) => {
						dbg!(err);
						Some(Err(()))
					},
					Ok(data) =>
						Some(client.send(Message::Text(data)).await)
				}
				
			}
		}
		
	}
	pub async fn send_soft(&self, id: &Id, message: OutgoingMessage) -> Option<()> {
		match self.try_send(id, message).await {
			Some(Ok(_)) => Some(()),
			_ => None
		}
	}
	pub async fn send(&self, id: &Id, message: OutgoingMessage) -> Option<()> {
		
		match self.try_send(id, message).await {
			Some(Ok(_)) => Some(()),
			Some(Err(_)) => None,
			_ => {
				println!("WebSocket send to invalid client [{}]", id);
				None
			}
		}
		
	}
	
}











