


use crate::Id;

pub use axum::extract::ws::{
	WebSocketUpgrade,
	WebSocket,
	Message
};

use futures_util::{
	//Future,
	sink::SinkExt,
	stream::{SplitSink, SplitStream, StreamExt}
};

use std::sync::Arc;
use dashmap::DashMap;

//use std::future::Future;

use serde::{Serialize, Deserialize};

#[derive(Debug)]
#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum IncomingMessage {
	Impression { receiver_id: String, liked: bool },
	ChatMessage { receiver_id: String, content: String }
}

#[derive(Debug)]
#[derive(Serialize)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum OutgoingMessage {
	Like,
	Match { user_id: String },
	ChatMessage { sender_id: String, content: String }
}



//use core::iter::FilterMap;
//pub type WebSocketReceiver<F> = FilterMap<SplitStream<WebSocket>, F>;
pub type WebSocketReceiver = SplitStream<WebSocket>;
pub type WebSocketSender = SplitSink<WebSocket, Message>;
//type OnMessage = Fn(&Client, Message) -> ();

#[derive(Clone)]
pub struct WebSocketState {
	clients: Arc<DashMap<Id, Client>>
}


struct Client {
	sender: WebSocketSender
}

impl WebSocketState {
	
	pub fn new() -> Self {
		Self {
			clients: Arc::new(DashMap::new()),
		}
	}
	
	pub async fn to_incoming(message: Result<Message, axum::Error>) -> Option<IncomingMessage> {
		
		//if let Some(message) = message {
		
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
		
		//}
		
		//None
		
	}
	
	//where
	//	F: Send + 'static + Fn(Self, Id, IncomingMessage) -> T,
	//	T: Send + 'static + Future<Output = ()>
	pub fn listen(&self, id: &Id, socket: WebSocket) -> WebSocketReceiver
		//FilterMap<WebSocketReceiver, F>
	{
		
		
		let (sender, receiver) = socket.split();
		
		let _old = self.clients.insert(id.clone(), Client { sender });
		
		/*
		if let Some(old) = old {
			// Do something? Does this matter?
		}
		*/
		
		receiver
		
		//receiver.try_filter(f)
		//receiver.filter_map(Self::filter_incoming)
		//receiver.filter_map(filter)
		
		/*
		receiver.filter_map(
			|message| match message {
				
			}
		)
		*/
		
		//receiver.filter_map(f)
		
		/*
		while let Some(message) = receiver.next().await {
			
			//println!("Message received: {:?}", message);
			
			match message {
				Err(err) => {
					println!("WebSocket receiver error: {}", err);
					break;
				}
				Ok(Message::Text(data)) => {
					
					let result = serde_json::from_str(&data);
					
					match result {
						Err(err) => println!("IncomingMessage deserialization error: {}", err),
						Ok(message) => {
							tokio::spawn(on_message(self.clone(), id.clone(), message));
						}
					}
					
				},
				Ok(Message::Close(_)) => {
					println!("WebSocket connection closed [{}]", id);
				},
				Ok(message) => {
					println!("WebSocket received invalid message type: {:?}", message);
					// break?
				}
			}
			
		}
		
		self.clients.remove(&id);
		*/
		
	}
	pub async fn drop(&self, id: &Id) {
		self.clients.remove(id);
	}
	
	pub async fn has(&self, id: &Id) -> bool {
		self.clients.contains_key(id)
	}
	
	pub async fn try_send(&self, id: &Id, message: OutgoingMessage) -> Option<Result<(), ()>> {
		
		if let Some(mut client) = self.clients.get_mut(id) {
			
			let result = serde_json::to_string(&message);
			
			match result {
				Err(err) => {
					println!("Message serialization error: {err}");
					Some(Err(()))
				},
				Ok(data) => {
					
					let message = Message::Text(data);
					let result = client.sender.send(message).await;
					
					match result {
						Ok(_) => Some(Ok(())),
						Err(err) => {
							println!("WebSocket send error: {}", err);
							//sender.close().await;
							Some(Err(()))
						}
					}
					
				}
			}
		} else {
			None
		}
		
	}
	pub async fn soft_send(&self, id: &Id, message: OutgoingMessage) -> Option<()> {
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











