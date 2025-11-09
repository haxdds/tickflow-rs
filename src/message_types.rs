// Importing types for asynchronous programming
use std::future::Future; // This brings in the Future trait, which represents values that might not be available yet (typical for async code)
use std::pin::Pin; // Pin is used for guaranteeing that the data won't be moved in memory (important for async/futures)

// This trait defines the requirements for a "Message" in this system.
// - `Send` means the message type can safely be sent across threads.
// - `Sync` means references to the message can safely be shared between threads.
// - `Clone` means we can make duplicates of our message type, which is useful if multiple parts of the program need to own or change a message.
// - `'static` means the type doesn't contain any non-static references (i.e., it can live as long as the program, good for async tasks).
pub trait Message: Send + Sync + Clone + 'static {}

// Here we create a type alias for batches of messages.
// Instead of always writing `Vec<M>`, we can write `MessageBatch<M>`.
// `M` is a generic type, so this can work with any message type that implements the `Message` trait.
pub type MessageBatch<M> = Vec<M>;

// This trait is for types that can "consume" or "handle" batches of messages.
// - The trait is generic over `M`, which must implement the `Message` trait.
// - `Send + Sync + 'static` means the sink itself can be shared and sent across threads, and is suitable for async.
pub trait MessageSink<M: Message>: Send + Sync + 'static {
    // This function returns the name of the sink as a string slice with a static lifetime.
    // Useful for logging, debugging, or identifying sinks.
    fn name(&self) -> &'static str;

    // This is the main function that actually handles a batch of messages.
    // - It takes an immutable reference to self (`&'a self`). The `'a` lifetime parameter ensures the future doesn't outlive the sink.
    // - It takes the batch of messages to handle.
    // - It returns a "pinned" boxed future.
    //      - `Pin<Box<...>>` means the future is stored on the heap and won't move in memory (important for async).
    //      - `dyn Future<Output = anyhow::Result<()>> + Send + 'a` means:
    //           - The future, when awaited, yields a `Result<(), anyhow::Error>` (so it's fallible).
    //           - The future can be sent between threads.
    //           - The future references the same lifetime as self.
    fn handle_batch<'a>(
        &'a self,
        batch: MessageBatch<M>,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>>;
}

// This trait is for types that generate or "source" messages—that is, feed messages to other parts of the system.
// - Like before, this is generic over `M: Message` and the source must be `Send` + `'static`.
pub trait MessageSource<M: Message>: Send + 'static {
    // This method runs the source so it can start sending messages to a receiver.
    // - `&'a mut self`: This takes a mutable reference to self for the duration of the operation, meaning the source can modify its own state.
    // - `tx`: This is a sender channel (from tokio's `mpsc`), over which batches of messages are sent.
    // - Like the sink, it returns a pinned boxed future that yields a `Result<(), anyhow::Error>`.
    //     - The future is `'a`, so it can reference self and must not outlive it.
    fn run<'a>(
        &'a mut self,
        tx: tokio::sync::mpsc::Sender<MessageBatch<M>>,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>>;
}
