//! Different methods of storing frames for transmission/recpetion
//!
//! For the most basic use cases the provided defaults will most likely be sufficient and this module can be ignored.

use transfer::TransferFrame;
use transfer::TransferFrameID;
use transfer::FullTransferID;
use transfer::TransferFrameIDFilter;

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
