use std::cell::RefCell;
use std::rc::{Rc, Weak};
use crate::system::Canvas;

#[derive(Debug,Clone)]
struct Node {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    name: String,
    parent: Option<Weak<RefCell<Node>>>,
    childs: Vec<Rc<RefCell<Node>>>,
}
#[derive(Debug,Clone)]
struct Widget(Rc<RefCell<Node>>);

impl Widget {
    pub fn new<S: Into<String>>(name: S, x: u32, y: u32, w: u32, h: u32, parent: Option<Widget>) -> Self {
        let p = parent.clone();
        let w = Rc::new(RefCell::new(Node {
            name: name.into(),
            x,
            y,
            w,
            h,
            parent: parent.map(|p| Rc::downgrade(&p.0)),
            childs: Vec::new(),
        }));
        if let Some(p) = p {
            p.0.borrow_mut().childs.push(w.clone());
        }
        Widget(w)
    }
}

pub trait Gadget {
    fn get_parent(&self) -> Option<Weak<RefCell<Self>>>;
    fn get_childs(&self) -> &[Rc<RefCell<Self>>];
    fn set_geometry(&mut self, x: u32, y: u32, w: u32, h: u32);
    fn get_geometry(&self) -> (u32,u32,u32,u32);
    fn draw(&mut self,canvas: &mut Canvas);
    fn click(&mut self, x:u32, y:u32) {
        for gadget in self.get_childs() {
            let geometry = self.get_geometry();
            if x >geometry.0 && y >geometry.1 && x<geometry.0 + geometry.2  && y<geometry.1 + geometry.3  {
                gadget.borrow_mut().click(x,y);
            }
        }
    }
}

struct Button {
    x: u32,y: u32, w:u32, h: u32,
    text: String,
    click: Option<fn (x: u32, y: u32) -> bool>
}


#[cfg(test)]
mod tests {
    use crate::widget::*;
    #[test]
    fn new_widget() {
        let root = Widget::new("root", 0, 0, 100, 100, None);
        let child = Widget::new("child", 0, 0, 100, 100, Some(root));

        let button = Button {
            x: 0, y: 0, w: 100, h:10,
            text: "Click me".to_owned(),
            click: Some(|x,y| {println!("clicked at {} {}",x,y); true}),
        };
    }
}
