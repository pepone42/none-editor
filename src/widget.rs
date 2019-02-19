// use crate::system::{Canvas,System};
// use glutin::WindowEvent;
// use id_tree::Tree;
// use std::rc::Rc;
// use std::cell::RefCell;

// #[derive(Debug, Clone, Copy)]
// pub enum Flow {
//     Horizontal,
//     Vertical,
// }

// pub enum EventResult<T> {
//     Redraw,
//     Continue,
//     Cancel,
//     Ok(T),
// }

// #[derive(Debug, Clone, Copy)]
// pub struct Rect {
//     x: f32,
//     y: f32,
//     w: f32,
//     h: f32,
// }

// impl Rect {
//     pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
//         Rect { x, y, w, h }
//     }
// }


// pub trait Widget<T: Sized> {
//     fn set_rect(&mut self, rect: Rect);
//     fn get_childs(&self) -> &[Rc<RefCell<dyn Widget<T>>>];
//     fn prefered_size(&self, canvas: &Canvas) -> (f32, f32);
//     fn reflow(&mut self, rect: Rect, canvas: &Canvas) {
//         self.set_rect(rect);
//         for child in self.get_childs() {
//             // TODO calculate the size
//             child.borrow_mut().reflow(rect, canvas);
//         }
//     }
//     fn draw(&mut self, canvas: &mut Canvas);
//     fn process_event<T>(&mut self, event: &WindowEvent) -> EventResult<T> {

//     }
// }




// struct Document {
//     root: Tree<Box<dyn Widget>>
// }



// fn run(mut system: System, widgets: Vec<Box<dyn Widget>>) {
//     let mut running = true;
//     let mut focused: usize = 0;
//     while running {
//         // process event
//         system.events_loop.poll_events(|event| {
//         });

//         // redraw

//     }
// }
// // #[derive(Debug, Clone)]
// // struct Node {
// //     rect: Rect,
// //     name: String,
// //     parent: Option<Weak<RefCell<Node>>>,
// //     childs: Vec<Rc<RefCell<Node>>>,
// //     flow: Flow

// // }
// // #[derive(Debug, Clone)]
// // struct Widget(Rc<RefCell<Node>>);

// // impl Widget {
// //     pub fn new<S: Into<String>>(name: S, rect: Rect, parent: Option<Widget>) -> Self {
// //         let p = parent.clone();
// //         let w = Rc::new(RefCell::new(Node {
// //             name: name.into(),
// //             rect,
// //             parent: parent.map(|p| Rc::downgrade(&p.0)),
// //             childs: Vec::new(),
// //         }));
// //         if let Some(p) = p {
// //             p.0.borrow_mut().childs.push(w.clone());
// //         }
// //         Widget(w)
// //     }
// // }

// #[cfg(test)]
// mod tests {
//     use crate::widget::*;
//     use id_tree::{Tree,Node,InsertBehavior};

//     struct Window {}
//         impl Widget for Window {
//         fn min_size(&self, canvas: &Canvas) -> (f32, f32) {
//             (10.0,10.0)
//         }
//         fn relayout(&mut self, rect: Rect, flow: Flow, canvas: &Canvas) {

//         }
//         fn draw(&mut self, canvas: &mut Canvas) {

//         }
//         fn process_event(&mut self, event: &WindowEvent) {

//         }
//     }

//     struct Button {

//     }
//     impl Widget for Button {
//         fn min_size(&self, canvas: &Canvas) -> (f32, f32) {
//             (10.0,10.0)
//         }
//         fn relayout(&mut self, rect: Rect, flow: Flow, canvas: &Canvas) {

//         }
//         fn draw(&mut self, canvas: &mut Canvas) {

//         }
//         fn process_event(&mut self, event: &WindowEvent) {

//         }
//     }

//     #[test]
//     fn new_widget() {
//         let mut root = Tree::<Box<dyn Widget>>::new();
//         let mut button: Node<Box<dyn Widget>> = Node::new(Box::new(Button{}));
//         root.insert(button,InsertBehavior::AsRoot);
//         // let root = Widget::new("root", Rect::new(0.0, 0.0, 100.0, 100.0), None);
//         // let child = Widget::new("child", Rect::new(0.0, 0.0, 100.0, 100.0), Some(root));
//     }
// }
