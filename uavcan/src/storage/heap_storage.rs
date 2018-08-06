use std::sync::{
    Mutex,
    Arc,
    Weak,
};

use std::collections::BinaryHeap;

use transfer::TransferFrame;
use transfer::TransferFrameID;
use transfer::FullTransferID;
use transfer::TransferFrameIDFilter;
use transfer::Priority;

use storage::Storage;
use storage::SubscriberStorageHandle;
use storage::InterfaceStorageHandle;
use storage::StorageError;

pub struct HeapStorage<F: TransferFrame> {
    subscriber_list: Mutex<Vec<SubscriberListEntry<F>>>,
    interface_list: Mutex<Vec<InterfaceListEntry<F>>>,
}

struct SubscriberListEntry<F: TransferFrame> {
    filter: TransferFrameIDFilter,
    storage: Weak<Mutex<Vec<F>>>,
}

struct InterfaceListEntry<F: TransferFrame> {
    storage: Weak<Mutex<BinaryHeap<Priority<F>>>>,
}

pub struct HeapSubscriberStorage<F> {
    storage: Arc<Mutex<Vec<F>>>,
}

pub struct HeapInterfaceStorage<F> {
    storage: Arc<Mutex < BinaryHeap < Priority < F > > > >,
}

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