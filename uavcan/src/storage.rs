//! Different methods of storing frames for transmission/recpetion
//!
//! For the most basic use cases the provided defaults will most likely be sufficient and this module can be ignored.

#[cfg(feature="std")]
use std::sync::{
    Mutex,
    Arc,
    Weak,
};

#[cfg(feature="std")]
use std::collections::BinaryHeap;

use transfer::TransferFrame;
use transfer::TransferFrameID;
use transfer::FullTransferID;
use transfer::TransferFrameIDFilter;
use transfer::Priority;

#[derive(Debug, PartialEq)]
pub enum StorageError {
    OutOfSpace,
}

pub trait Storage<F: TransferFrame> {
    type SubscriberStorageHandle: SubscriberStorageHandle<F>;
    type InterfaceStorageHandle: InterfaceStorageHandle<F>;

    /// Create a new storage container.
    fn new() -> Self;

    /// Create a subscription on all frames matching a filter.
    fn subscribe_to(&self, filter: TransferFrameIDFilter) -> Self::SubscriberStorageHandle;

    /// Creates an interface queue for a new interface.
    fn new_interface(&self) -> Self::InterfaceStorageHandle;

    /// Insert a frame to storage and route it to the correct subscribers.
    ///
    /// If there are no relevant subscribers `frame` will be dropped.
    /// If there are multiple relevant subscribers `frame` will be routed to all of them.
    fn insert_subscriber_queue(&self, frame: F) -> Result<(), StorageError>;

    /// Insert a frame to storage and route it to the interface for transmission.
    ///
    /// If there are multiple interface storage queues the frame will be added to all of them.
    fn insert_interface_queue(&self, frame: F) -> Result<(), StorageError>;
}

pub trait SubscriberStorageHandle<F: TransferFrame> {
    /// Remove and return the next frame matching the indentifier if such frame exist.
    ///
    /// It's important that `receive` returns frames in the correct order.
    fn remove(&self, identifier: &TransferFrameID) -> Option<F>;

    /// Finds the first element matching the predicate and returns its `TransferFrameID`.
    fn find_id<P>(&self, predicate: P) -> Option<FullTransferID>
        where P: FnMut(&F) -> bool;

    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all elements e such that `f(&e)` returns false.
    /// This method must operate in place and preserves the order of the retained elements.
    fn retain<P>(&self, predicate: P)
        where P: FnMut(&F) -> bool;
}

pub trait InterfaceStorageHandle<F: TransferFrame> {
    /// Removes the item with highest priority from the priority queue and returns it, or `None` if it is empty.
    fn pop(&self) -> Option<F>;

    /// Returns the `TransferFrameID` of the `TransferFrame` with highest priority, or `None` if the queue is empty.
    fn max_priority(&self) -> Option<TransferFrameID>;

    /// Pushes an item on the interface queue.
    ///
    /// This is the same as calling `insert_interface_queue` on the `Storage` which this handle is associated with.
    fn push(&self, frame: F) -> Result<(), StorageError>;
}


#[cfg(feature="std")]
pub struct HeapStorage<F: TransferFrame> {
    subscriber_list: Mutex<Vec<SubscriberListEntry<F>>>,
    interface_list: Mutex<Vec<InterfaceListEntry<F>>>,
}

#[cfg(feature="std")]
struct SubscriberListEntry<F: TransferFrame> {
    filter: TransferFrameIDFilter,
    storage: Weak<Mutex<Vec<F>>>,
}

#[cfg(feature="std")]
struct InterfaceListEntry<F: TransferFrame> {
    storage: Weak<Mutex<BinaryHeap<Priority<F>>>>,
}

#[cfg(feature="std")]
pub struct HeapSubscriberStorage<F> {
    storage: Arc<Mutex<Vec<F>>>,
}

#[cfg(feature="std")]
pub struct HeapInterfaceStorage<F> {
    storage: Arc<Mutex < BinaryHeap < Priority < F > > > >,
}

#[cfg(feature="std")]
impl<F: TransferFrame + Clone> Storage<F> for HeapStorage<F> {
    type SubscriberStorageHandle = HeapSubscriberStorage<F>;
    type InterfaceStorageHandle = HeapInterfaceStorage<F>;

    fn new() -> Self {
        HeapStorage {
            subscriber_list: Mutex::new(Vec::new()),
            interface_list: Mutex::new(Vec::new()),
        }
    }

    fn subscribe_to(&self, filter: TransferFrameIDFilter) -> Self::SubscriberStorageHandle {
        let storage = Arc::new(Mutex::new(Vec::new()));

        let subsciber_list_entry = SubscriberListEntry {
            filter,
            storage: Arc::downgrade(&storage),
        };

        let subscriber_handle = HeapSubscriberStorage {
            storage,
        };

        // TODO: attempt to replace old (weak pointers that turns into None) list entries before creating new
        self.subscriber_list.lock().unwrap().push(subsciber_list_entry);
        subscriber_handle
    }

    fn new_interface(&self) -> Self::InterfaceStorageHandle {
        let storage = Arc::new(Mutex::new(BinaryHeap::new()));

        let interface_list_entry = InterfaceListEntry {
            storage: Arc::downgrade(&storage),
        };

        let interface_handle = HeapInterfaceStorage {
            storage,
        };

        self.interface_list.lock().unwrap().push(interface_list_entry);
        interface_handle
    }


    fn insert_subscriber_queue(&self, frame: F) -> Result<(), StorageError> {
        for storage in self.subscriber_list.lock().unwrap().iter().filter(|x| x.filter.is_match(frame.id())).filter_map(|x| x.storage.upgrade()) {
            storage.lock().unwrap().push(frame.clone());
        }
        Ok(())
    }

    fn insert_interface_queue(&self, frame: F) -> Result<(), StorageError> {
        for storage in self.interface_list.lock().unwrap().iter().filter_map(|x| x.storage.upgrade()) {
            storage.lock().unwrap().push(Priority(frame.clone()));
        }
        Ok(())
    }
}

#[cfg(feature="std")]
impl<F: TransferFrame> SubscriberStorageHandle<F> for HeapSubscriberStorage<F> {
    fn remove(&self, identifier: &TransferFrameID) -> Option<F> {
        let mut queue = self.storage.lock().unwrap();
        let pos = queue.iter().position(|x| x.id() == *identifier)?;
        Some(queue.remove(pos))
    }

    fn find_id<P>(&self, mut predicate: P) -> Option<FullTransferID>
        where P: FnMut(&F) -> bool {
        Some(self.storage.lock().unwrap().iter().find(|x: &&F| predicate(*x))?.full_id())
    }

    fn retain<P>(&self, predicate: P)
        where P: FnMut(&F) -> bool {
        self.storage.lock().unwrap().retain(predicate)
    }
}

#[cfg(feature="std")]
impl<F: TransferFrame> InterfaceStorageHandle<F> for HeapInterfaceStorage<F> {
    fn pop(&self) -> Option<F> {
        Some(self.storage.lock().unwrap().pop()?.0)
    }

    fn max_priority(&self) -> Option<TransferFrameID> {
        Some(self.storage.lock().unwrap().peek()?.0.id())
    }

    fn push(&self, frame: F) -> Result<(), StorageError> {
        self.storage.lock().unwrap().push(Priority(frame));
        Ok(())
    }
}
