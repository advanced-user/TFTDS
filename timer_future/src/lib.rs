mod executor;
mod timer_future;

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::{executor::{new_executor_and_spawner, Spawner}, timer_future::TimerFuture};

    #[test] 
    fn test_executor() {
        let (executor, spawner) = new_executor_and_spawner();

        let c = async {
            println!("run async without await!");
        };

        // Spawn a task to print before and after waiting on a timer.
        spawner.spawn(async {
            println!("run async!");
            // Wait for our timer future to complete after two seconds.
            TimerFuture::new(Duration::new(2, 0)).await;
            println!("done!");
        });
        
        spawner.spawn(async {
            println!("run async 2!");
            // Wait for our timer future to complete after two seconds.
            TimerFuture::new(Duration::new(2, 0)).await;
            println!("done 2!");
        });

        println!("contionue");

        // Drop the spawner so that our executor knows it is finished and won't
        // receive more incoming tasks to run.
        drop(spawner);
    
        // Run the executor until the task queue is empty.
        // This will print "howdy!", pause, and then print "done!".
        executor.run();
    }

    fn drop(_: Spawner) {}
}
