use std::sync::Arc;

use anyhow::{anyhow, Result};
use tokio::sync::{mpsc, Mutex};

use tickflow::core::{Message, MessageBatch, MessageSink, MessageSource};
use tickflow::pipeline::{MessageProcessor, SPSCDataFeed};

#[derive(Debug, Clone, PartialEq, Eq)]
struct TestMessage(&'static str);

impl Message for TestMessage {}

#[derive(Clone)]
struct MockSource {
    batches: Vec<MessageBatch<TestMessage>>,
    fail_at: Option<usize>,
}

impl MockSource {
    fn new(batches: Vec<MessageBatch<TestMessage>>) -> Self {
        Self {
            batches,
            fail_at: None,
        }
    }

    fn fail_at(mut self, index: usize) -> Self {
        self.fail_at = Some(index);
        self
    }
}

impl MessageSource<TestMessage> for MockSource {
    fn run<'a>(
        &'a mut self,
        tx: mpsc::Sender<MessageBatch<TestMessage>>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            let fail_at = self.fail_at;
            for (idx, batch) in std::mem::take(&mut self.batches).into_iter().enumerate() {
                if Some(idx) == fail_at {
                    return Err(anyhow!("mock source failure at batch {idx}"));
                }
                tx.send(batch)
                    .await
                    .map_err(|err| anyhow!("send failed: {err}"))?;
            }
            Ok(())
        })
    }
}

#[derive(Default)]
struct MockSinkState {
    batches: Vec<MessageBatch<TestMessage>>,
    failures_remaining: usize,
}

#[derive(Clone, Default)]
struct MockSink {
    name: &'static str,
    state: Arc<Mutex<MockSinkState>>,
}

impl MockSink {
    fn new(name: &'static str) -> Self {
        Self {
            name,
            state: Arc::new(Mutex::new(MockSinkState::default())),
        }
    }

    fn with_failures(name: &'static str, failures: usize) -> Self {
        Self {
            name,
            state: Arc::new(Mutex::new(MockSinkState {
                batches: Vec::new(),
                failures_remaining: failures,
            })),
        }
    }

    async fn handled_batches(&self) -> Vec<MessageBatch<TestMessage>> {
        let state = self.state.lock().await;
        state.batches.clone()
    }
}

impl MessageSink<TestMessage> for MockSink {
    fn name(&self) -> &'static str {
        self.name
    }

    fn handle_batch<'a>(
        &'a self,
        batch: MessageBatch<TestMessage>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            let mut state = self.state.lock().await;
            state.batches.push(batch);
            if state.failures_remaining > 0 {
                state.failures_remaining -= 1;
                Err(anyhow!("mock sink failure"))
            } else {
                Ok(())
            }
        })
    }
}

#[tokio::test]
async fn tickflow_builder_start_processes_batches() {
    let source_batches = vec![
        vec![TestMessage("alpha"), TestMessage("beta")],
        vec![TestMessage("gamma")],
    ];
    let source = MockSource::new(source_batches);
    let sink = MockSink::new("collector");

    let handles = SPSCDataFeed::builder(source, sink.clone())
        .channel_capacity(4)
        .start()
        .await
        .expect("failed to start data feed");

    handles
        .source
        .await
        .expect("source task panicked unexpectedly");
    handles
        .processor
        .await
        .expect("processor task panicked unexpectedly");

    let batches = sink.handled_batches().await;
    assert_eq!(batches.len(), 2);
    assert_eq!(
        batches[0],
        vec![TestMessage("alpha"), TestMessage("beta")]
    );
    assert_eq!(batches[1], vec![TestMessage("gamma")]);
}

#[tokio::test]
async fn message_processor_continues_after_sink_errors() {
    let sink = MockSink::with_failures("flaky", 1);
    let processor = MessageProcessor::new(sink.clone());
    let (tx, rx) = mpsc::channel(4);

    let processor_task = tokio::spawn(async move { processor.process_messages(rx).await });

    tx.send(vec![TestMessage("first")])
        .await
        .expect("send first batch");
    tx.send(vec![TestMessage("second")])
        .await
        .expect("send second batch");
    drop(tx);

    processor_task
        .await
        .expect("processor join failure")
        .expect("processor returned error");

    let batches = sink.handled_batches().await;
    assert_eq!(batches.len(), 2);
    assert_eq!(batches[0], vec![TestMessage("first")]);
    assert_eq!(batches[1], vec![TestMessage("second")]);
}

#[tokio::test]
async fn datafeed_completes_when_source_fails_midstream() {
    let source_batches = vec![vec![TestMessage("only")], vec![TestMessage("never sent")]];
    let source = MockSource::new(source_batches).fail_at(1);
    let sink = MockSink::new("collector");

    let handles = SPSCDataFeed::builder(source, sink.clone())
        .channel_capacity(2)
        .start()
        .await
        .expect("failed to start data feed");

    handles
        .source
        .await
        .expect("source task panicked unexpectedly");
    handles
        .processor
        .await
        .expect("processor task panicked unexpectedly");

    let batches = sink.handled_batches().await;
    assert_eq!(batches.len(), 1);
    assert_eq!(batches[0], vec![TestMessage("only")]);
}

