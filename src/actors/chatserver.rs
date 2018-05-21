// `ChatServer` is an actor. It maintains list of connection client session.
// And manages available rooms. Peers send messages to other peers in same
// room through `ChatServer`.

use actix::prelude::*;
use rand::{self, Rng, ThreadRng};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

// Message for chat server communications
#[derive(Message)]
pub struct SessionMessage(pub String);

// New chat session is created
#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub addr: Recipient<Syn, SessionMessage>,
}

// Session is disconnected
#[derive(Message)]
pub struct Disconnect {
    pub id: usize,
}

// Send message to specific room
#[derive(Message)]
pub struct Message {
    // Id of the client session
    pub id: usize,
    // Peer message
    pub msg: String,
    // Room name
    pub room: String,
}

// `ChatServer` manages chat rooms and responsible for coordinating chat session.
// implementation is super primitive
pub struct ChatServer {
    sessions: HashMap<usize, Recipient<Syn, SessionMessage>>,
    rooms: HashMap<String, HashSet<usize>>,
    rng: RefCell<ThreadRng>,
}

impl Default for ChatServer {
    fn default() -> ChatServer {
        // default room
        let mut rooms = HashMap::new();
        rooms.insert("Main".to_owned(), HashSet::new());

        ChatServer {
            sessions: HashMap::new(),
            rooms: rooms,
            rng: RefCell::new(rand::thread_rng()),
        }
    }
}

impl ChatServer {
    // Send message to all users in the room
    fn send_message(&self, room: &str, message: &str) {
        if let Some(sessions) = self.rooms.get(room) {
            for id in sessions {
                if let Some(addr) = self.sessions.get(id) {
                    let _ = addr.do_send(SessionMessage(message.to_owned()));
                }
            }
        }
    }
}

// Make actor from `ChatServer`
impl Actor for ChatServer {
    // We are going to use simple Context, we just need ability to communicate
    // with other actors.
    type Context = Context<Self>;
}

// Handler for Connect message.
//
// Register new session and assign unique id to this session
impl Handler<Connect> for ChatServer {
    type Result = usize;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        println!("Someone joined");

        let joinmsg = "{\"message\": \"someone joined\", \"name\":\"server\"}";
        //let joinmsg = json::to_string(&msg.0).unwrap();
        // notify all users in same room
        self.send_message(&"Main".to_owned(), &joinmsg);

        // register session with random id
        let id = self.rng.borrow_mut().gen::<usize>();
        self.sessions.insert(id, msg.addr);

        // auto join session to Main room
        self.rooms.get_mut(&"Main".to_owned()).unwrap().insert(id);

        // send id back
        id
    }
}

// Handler for Disconnect message.
impl Handler<Disconnect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        println!("Someone disconnected");

        let dcmsg = "{\"message\": \"someone disconnected\", \"name\":\"server\"}";
        let mut rooms: Vec<String> = Vec::new();

        // remove address
        if self.sessions.remove(&msg.id).is_some() {
            // remove session from all rooms
            for (name, sessions) in &mut self.rooms {
                if sessions.remove(&msg.id) {
                    rooms.push(name.to_owned());
                }
            }
        }
        // send message to other users
        for room in rooms {
            self.send_message(&room, &dcmsg);
        }
    }
}

// Handler for Message message.
impl Handler<Message> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Message, _: &mut Context<Self>) {
        self.send_message(&msg.room, msg.msg.as_str());
    }
}
