use std::{cell::RefCell, rc::Rc};

use bounce::{use_atom, Atom, UseAtomHandle};
use gloo_worker::{Bridge, Bridged, Callback, Worker};
use yew::use_effect_with_deps;

use crate::worker::WordleWorker;

#[derive(Atom)]
pub struct WordleWorkerAtom(Rc<RefCell<Box<dyn Bridge<WordleWorker>>>>);

impl WordleWorkerAtom {
    pub fn send(&self, msg: <WordleWorker as Worker>::Input) {
        self.0.borrow_mut().send(msg);
    }

    fn set_callback(&self, cb: Callback<<WordleWorker as Worker>::Output>) {
        *self.0.borrow_mut() = WordleWorker::bridge(cb);
    }

    pub fn with_callback(cb: Callback<<WordleWorker as Worker>::Output>) -> UseAtomHandle<Self> {
        let worker = use_atom::<WordleWorkerAtom>();

        {
            let worker = worker.clone();
            use_effect_with_deps(
                move |_| {
                    log::info!("initialize worker with callback");
                    worker.set_callback(cb);
                    || ()
                },
                (),
            );
        }

        worker
    }
}

impl PartialEq for WordleWorkerAtom {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Default for WordleWorkerAtom {
    fn default() -> Self {
        log::info!("default");
        let cb = |_| ();
        Self(Rc::new(RefCell::new(WordleWorker::bridge(Rc::new(cb)))))
    }
}
