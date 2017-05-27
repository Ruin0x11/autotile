use std::ops::Deref;
use std::rc::Rc;

use glium::glutin;

use ui::Ui;
use ui::elements::*;

pub trait UiLayer: UiElement {
    fn on_event(&mut self, event: glutin::Event) -> EventResult;
}

pub trait UiQuery: UiLayer {
    type QueryResult;

    fn result(&self) -> Self::QueryResult;
}

pub struct Callback(Rc<Box<Fn(&mut Ui)>>);

impl Callback {
    pub fn from_fn<F: Fn(&mut Ui) + 'static>(f: F) -> Self {
        Callback(Rc::new(Box::new(f)))
    }
}

impl Deref for Callback {
    type Target = Box<Fn(&mut Ui)>;
    fn deref<'a>(&'a self) -> &'a Box<Fn(&mut Ui)> {
        &self.0
    }
}

impl From<Rc<Box<Fn(&mut Ui)>>> for Callback {
    fn from(f: Rc<Box<Fn(&mut Ui)>>) -> Self {
        Callback(f)
    }
}

impl From<Box<Fn(&mut Ui) + Send>> for Callback {
    fn from(f: Box<Fn(&mut Ui) + Send>) -> Self {
        Callback(Rc::new(f))
    }
}

impl From<Box<Fn(&mut Ui)>> for Callback {
    fn from(f: Box<Fn(&mut Ui)>) -> Self {
        Callback(Rc::new(f))
    }
}

pub enum EventResult {
    Ignored,
    Consumed(Option<Callback>),
    Done
}
