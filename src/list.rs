use crate::sync::atomic::AtomicPtr;
use crate::{BoxedPointer, Doer, Holder};
use std::marker::PhantomData;
use std::sync::atomic::Ordering;

static DROPBOX: BoxedPointer = BoxedPointer::new();

pub(crate) struct Node<T> {
    pub(crate) value: T,
    pub(crate) next: AtomicPtr<Node<T>>,
}

impl<T: Clone> Node<T> {
    pub(crate) fn new(value: T) -> Self {
        Self {
            value,
            next: AtomicPtr::new(std::ptr::null_mut()),
        }
    }
}

pub struct Stack<T> {
    pub(crate) head: AtomicPtr<Node<T>>,
    marker: PhantomData<Node<T>>,
}

unsafe impl<T> Send for Stack<T> where T: Send {}
unsafe impl<T> Sync for Stack<T> where T: Sync {}

impl<T> Drop for Stack<T> {
    fn drop(&mut self) {
        let mut current = self.head.load(Ordering::SeqCst);
        while !current.is_null() {
            let next = unsafe { (*current).next.load(Ordering::SeqCst) };
            let owned = unsafe { Box::from_raw(current) };
            std::mem::drop(owned);
            current = next;
        }
        Holder::try_reclaim();
    }
}

impl<T: Clone> Stack<T> {
    pub fn new() -> Self {
        Self {
            head: AtomicPtr::new(std::ptr::null_mut()),
            marker: PhantomData,
        }
    }

    pub fn insert<'a>(&self, value: T) -> Result<&str, &str> {
        let mut attempts = 0;
        loop {
            if attempts > 15 {
                return Err("Insertion failed. Try again!");
            }
            let mut holder = Holder::default();
            let guard = unsafe { holder.load_pointer(&self.head) };
            let current_head = if let Some(ref guard) = guard {
                guard.data
            } else {
                std::ptr::null_mut()
            };
            let new_node = Node::new(value.clone());
            new_node.next.store(current_head, Ordering::SeqCst);
            let boxed = Box::into_raw(Box::new(new_node));
            if self
                .head
                .compare_exchange(current_head, boxed, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                Holder::try_reclaim();
                return Ok("Insertion successful!");
            } else {
                let owned = unsafe { Box::from_raw(boxed) };
                std::mem::drop(owned);
                attempts += 1;
            }
        }
    }

    pub fn delete<'a>(&self) -> Result<T, &str> {
        let mut attempts = 0;
        loop {
            if attempts > 15 {
                return Err("Deletion failed. Try again!");
            }
            let mut holder = Holder::default();
            let guard = unsafe { holder.load_pointer(&self.head) };
            let current_head = if let Some(ref guard) = guard {
                guard.data
            } else {
                std::ptr::null_mut()
            };
            if current_head.is_null() {
                Holder::try_reclaim();
                return Err("There are no elements in the list");
            }
            let next_head = unsafe { (*current_head).next.load(Ordering::SeqCst) };
            if self
                .head
                .compare_exchange(current_head, next_head, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                let value = unsafe { std::ptr::read(&(*current_head).value) };
                let mut holder = Holder::default();
                let wrapper =
                    unsafe { holder.get_wrapper(&AtomicPtr::new(current_head), &DROPBOX) };
                wrapper.expect("Has to be there").retire();
                Holder::try_reclaim();
                return Ok(value);
            } else {
                attempts += 1;
            }
        }
    }
}
