

use crate::Id;
use crate::models::{
	
	Profile,
	ChatMessage,
	
	Sender
};
use serde::{Serialize, Deserialize};



#[derive(Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct InitialMatchData {
	profiles: Vec<Profile>,
	messages: Vec<RemoteChatMessage>
}
impl InitialMatchData {
	
	pub fn new(profiles: Vec<Profile>, messages: Vec<ChatMessage>, for_user: &Id) -> Self {
		Self {
			profiles,
			messages: RemoteChatMessage::new_vec(messages, for_user)
		}
	}
	
}


//#[derive(Selectable)]
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct RemoteChatMessage {
	
	pub id: String,
	pub user: String,
	
	pub outgoing: bool,
	pub timestamp: String,
	
	pub content: String
	
}
impl RemoteChatMessage {
	
	pub fn new(message: ChatMessage, for_user: &Id) -> Self {
		
		let for_user = &**for_user;
		
		let other_user;
		let outgoing;
		
		if for_user == &message.user1 {
			other_user = message.user2;
			outgoing = message.sender == Sender::One;
		} else if for_user == &message.user2 {
			other_user = message.user1;
			outgoing = message.sender == Sender::Two;
		} else {
			println!("Invalid for_user for RemoteChatMessage");
			other_user = message.user1;
			outgoing = false;
		}
		
		RemoteChatMessage {
			id: message.id,
			user: other_user,
			timestamp: message.timestamp,
			content: message.content,
			outgoing
		}
		
	}
	pub fn new_vec(messages: Vec<ChatMessage>, for_user: &Id) -> Vec<Self> {
		
		messages
			.into_iter()
			.map(|message| Self::new(message, for_user))
			.collect()
		
	}
	
}





