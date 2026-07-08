use tokio::runtime::Runtime;

pub struct ModulePool {
    worker_threads: usize,
    thread_pool: Option<Runtime>
}

impl ModulePool {

pub fn new(
    worker_threads: usize,
) -> Self {
    Self 
        {
            worker_threads,
            thread_pool: None
        }
    }

    pub fn build(&mut self) {
        // Build async runtime of thread pool
        let rt = tokio::runtime::Builder::new_multi_thread()
            .name("module-pool")
            .worker_threads(self.worker_threads)
            .enable_all()
            .build()
            .expect("Failed to build module thread pool runtime");
        self.thread_pool = Some(rt);
    }

    pub fn add<Fut>(&self, f: Fut) 
    where
        Fut: Future<Output = ()> + Send + 'static
    {
        match &self.thread_pool {
            Some(rt) => {
                rt.spawn(f);
            },
            None => eprintln!("Runtime not built yet")
        };
    }
}
