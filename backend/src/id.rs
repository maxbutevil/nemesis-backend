

use internment::ArcIntern;
pub type Id = ArcIntern<String>;


/*
use internment::ArcIntern;
pub type Id = ArcIntern<String>;


#[derive(Clone, Copy)]
struct Id(ArcIntern<String>);

impl Id {
	
	fn new(id: String) -> Self {
		Self(ArcIntern::new(id))
	}
	
}

impl std::ops::Deref for Id {
	
	type Target = String;
	
	fn deref(&self) -> &String {
		&*self.0
	}
	
}
*/

