


use crate::Id;

pub use axum::extract::ws::{
	WebSocketUpgrade,
	WebSocket,
	Message
};

use futures_util::{
	sink::SinkExt,
	stream::{
		StreamExt,
		SplitSink,
		SplitStream
	}
};

use std::sync::Arc;
use dashmap::DashMap;

//use std::future::Future;



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
	
	pub async fn handle_socket<'a, T, F>(self, id: Id, socket: WebSocket, mut on_message: F)
	where
		F: Send + 'static + FnMut(Self, Id, Message) -> T,
		//T: Send + 'static + Future<Output = ()>
	{
		
		let (sender, mut receiver) = socket.split();
		self.clients.insert(id.clone(), Client { sender });
		
		while let Some(message) = receiver.next().await {
			
			match message {
				Err(err) => {
					println!("WebSocket receiver error: {}", err);
					break;
				}
				Ok(message) => {
					on_message(self.clone(), id.clone(), message);
				}
			}
			
		}
		
		self.clients.remove(&id);
		
	}
	
	pub async fn has(&self, id: &Id) -> bool {
		self.clients.contains_key(id)
	}
	
	pub async fn try_send(&self, id: &Id, message: Message) -> Option<Result<(), ()>> {
		
		if let Some(mut client) = self.clients.get_mut(id) {
			
			let result = client.sender.send(message).await;
			
			match result {
				Ok(_) => Some(Ok(())),
				Err(err) => {
					println!("WebSocket send error: {}", err);
					//sender.close().await;
					Some(Err(()))
				}
			}
			
		} else {
			None
		}
		
	}
	pub async fn soft_send(&self, id: &Id, message: Message) -> Option<()> {
		
		match self.try_send(id, message).await {
			Some(Ok(_)) => Some(()),
			_ => None
		}
		
	}
	pub async fn send(&self, id: &Id, message: Message) -> Option<()> {
		
		match self.try_send(id, message).await {
			Some(result) => result.ok(),
			_ => {
				println!("WebSocket send to invalid client [{}]", id);
				None
			}
		}
		
	}
	
}











