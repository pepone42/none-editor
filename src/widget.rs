use std::cell::RefCell;
use std::rc::{Rc, Weak};

#[derive(Debug,Clone)]
struct Node {
    w: u32,
    h: u32,
    name: String,
    parent: Option<Weak<RefCell<Node>>>,
    childs: Vec<Rc<RefCell<Node>>>,
}
#[derive(Debug,Clone)]
struct Widget(Rc<RefCell<Node>>);

impl Widget {
    pub fn new<S: Into<String>>(name: S, w: u32, h: u32, parent: Option<Widget>) -> Self {
        let p = parent.clone();
        let w = Rc::new(RefCell::new(Node {
            name: name.into(),
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

#[cfg(test)]
mod tests {
    use crate::widget::*;
    #[test]
    fn new_widget() {
        let root = Widget::new("root", 100, 100, None);
        let child = Widget::new("child", 100, 100, Some(root));
    }
}
