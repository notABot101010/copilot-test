use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub title: String,
    pub body: String,
}

impl Document {
    pub fn new(title: String) -> Self {
        Self {
            title,
            body: String::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum NodeColor {
    Default,
    Red,
    Green,
    Blue,
    Yellow,
    Magenta,
    Cyan,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: Uuid,
    pub document: Document,
    pub x: f64,
    pub y: f64,
    pub width: u16,
    pub height: u16,
    pub color: NodeColor,
}

impl Node {
    pub fn new(title: String, x: f64, y: f64) -> Self {
        Self {
            id: Uuid::new_v4(),
            document: Document::new(title),
            x,
            y,
            width: 20,
            height: 3,
            color: NodeColor::Default,
        }
    }

    pub fn contains_point(&self, world_x: f64, world_y: f64) -> bool {
        world_x >= self.x 
            && world_x < self.x + self.width as f64 
            && world_y >= self.y 
            && world_y < self.y + self.height as f64
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub from: Uuid,
    pub to: Uuid,
}

impl Connection {
    pub fn new(from: Uuid, to: Uuid) -> Self {
        Self { from, to }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MindMap {
    pub nodes: Vec<Node>,
    pub connections: Vec<Connection>,
}

impl MindMap {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            connections: Vec::new(),
        }
    }

    pub fn add_node(&mut self, node: Node) {
        self.nodes.push(node);
    }

    pub fn remove_node(&mut self, id: Uuid) {
        self.nodes.retain(|n| n.id != id);
        self.connections.retain(|c| c.from != id && c.to != id);
    }

    pub fn add_connection(&mut self, from: Uuid, to: Uuid) {
        // Don't add duplicate connections
        if !self.connections.iter().any(|c| c.from == from && c.to == to) {
            self.connections.push(Connection::new(from, to));
        }
    }

    pub fn remove_connection(&mut self, from: Uuid, to: Uuid) {
        self.connections.retain(|c| !(c.from == from && c.to == to));
    }

    pub fn find_node_at(&self, world_x: f64, world_y: f64) -> Option<usize> {
        // Search in reverse order so top nodes are selected first
        for (i, node) in self.nodes.iter().enumerate().rev() {
            if node.contains_point(world_x, world_y) {
                return Some(i);
            }
        }
        None
    }

    pub fn get_node_by_id(&self, id: Uuid) -> Option<&Node> {
        self.nodes.iter().find(|n| n.id == id)
    }

    pub fn get_node_by_id_mut(&mut self, id: Uuid) -> Option<&mut Node> {
        self.nodes.iter_mut().find(|n| n.id == id)
    }
}
