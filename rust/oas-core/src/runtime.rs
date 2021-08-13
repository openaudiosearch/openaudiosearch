use futures::{stream::FuturesUnordered, Future, StreamExt};
use tokio::task::JoinHandle;

#[derive(Default)]
pub struct Runtime {
    // inner: Arc<Mutex<RuntimeInner>>
    tasks: Vec<JoinHandle<()>>,
}

impl Runtime {
    pub fn new() -> Self {
        Default::default()
    }

    pub async fn run(self) {
        let mut futs: FuturesUnordered<_> = self.tasks.into_iter().collect();
        while let Some(_result) = futs.next().await {
            // Nothing to do.
        }
    }

    pub fn spawn<T, S>(&mut self, label: S, future: T)
    where
        S: ToString,
        T: Future<Output = anyhow::Result<()>> + Send + 'static,
    {
        let label = label.to_string();
        let join_handle = tokio::task::spawn(async move {
            let res = future.await;
            match res {
                Ok(()) => {}
                Err(err) => {
                    log::error!("Task \"{}\" failed: {:?}", label, err);
                }
            }
        });
        self.tasks.push(join_handle);
    }
}
