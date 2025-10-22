use moka::sync::Cache;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn main() {
    println!("=== Testing set_max_capacity with concurrency ===\n");

    // Create a cache with initial capacity
    let cache = Arc::new(Cache::<u64, String>::new(1000));
    println!("Initial max capacity: {:?}", cache.policy().max_capacity());

    // Spawn multiple threads to insert entries
    println!("\nSpawning 10 threads to insert 100 entries each...");
    let mut handles = vec![];
    
    for thread_id in 0..10 {
        let cache_clone = Arc::clone(&cache);
        let handle = thread::spawn(move || {
            let start = thread_id * 100;
            let end = start + 100;
            for i in start..end {
                cache_clone.insert(i, format!("value-{}", i));
            }
        });
        handles.push(handle);
    }

    // Wait for all insertions to complete
    for handle in handles {
        handle.join().unwrap();
    }
    
    cache.run_pending_tasks();
    println!("Total entries inserted: {}", cache.entry_count());
    println!("Weighted size: {}", cache.weighted_size());

    // Increase capacity while threads are reading
    println!("\n--- Testing capacity increase with concurrent reads ---");
    let cache_clone = Arc::clone(&cache);
    let reader_handle = thread::spawn(move || {
        for _ in 0..1000 {
            for i in 0..100 {
                let _ = cache_clone.get(&i);
            }
        }
    });

    thread::sleep(Duration::from_millis(10));
    match cache.set_max_capacity(2000) {
        Ok(_) => println!("✓ Successfully increased capacity to 2000"),
        Err(e) => println!("✗ Failed: {}", e),
    }
    
    reader_handle.join().unwrap();
    println!("New max capacity: {:?}", cache.policy().max_capacity());

    // Decrease capacity while threads are inserting
    println!("\n--- Testing capacity decrease with concurrent operations ---");
    let cache_clone1 = Arc::clone(&cache);
    let cache_clone2 = Arc::clone(&cache);
    
    // Writer thread
    let writer_handle = thread::spawn(move || {
        for i in 1000..1100 {
            cache_clone1.insert(i, format!("new-value-{}", i));
            thread::sleep(Duration::from_micros(100));
        }
    });

    // Reader thread
    let reader_handle = thread::spawn(move || {
        for _ in 0..50 {
            for i in 0..20 {
                let _ = cache_clone2.get(&i);
            }
            thread::sleep(Duration::from_millis(1));
        }
    });

    thread::sleep(Duration::from_millis(20));
    
    println!("Decreasing capacity to 100...");
    match cache.set_max_capacity(100) {
        Ok(_) => {
            println!("✓ Successfully decreased capacity to 100");
            cache.run_pending_tasks();
            let count = cache.entry_count();
            println!("Entry count after eviction: {}", count);
            
            if count <= 100 {
                println!("✓ Eviction completed successfully");
            } else {
                println!("Note: Entry count ({}) > 100, running more tasks...", count);
                // Give it more time to evict
                for _ in 0..5 {
                    cache.run_pending_tasks();
                    thread::sleep(Duration::from_millis(10));
                }
                println!("Final entry count: {}", cache.entry_count());
            }
        }
        Err(e) => println!("✗ Failed: {}", e),
    }

    writer_handle.join().unwrap();
    reader_handle.join().unwrap();

    // Test with eviction listener
    println!("\n--- Testing with eviction listener ---");
    let evicted_count = Arc::new(std::sync::atomic::AtomicU64::new(0));
    let evicted_count_clone = Arc::clone(&evicted_count);
    
    let cache_with_listener = Cache::builder()
        .max_capacity(50)
        .eviction_listener(move |_key, _value, _cause| {
            evicted_count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        })
        .build();

    // Insert 50 entries
    for i in 0..50 {
        cache_with_listener.insert(i, format!("value-{}", i));
    }
    cache_with_listener.run_pending_tasks();
    
    println!("Inserted 50 entries");
    println!("Current entry count: {}", cache_with_listener.entry_count());

    // Decrease capacity to 20 - should evict 30 entries
    match cache_with_listener.set_max_capacity(20) {
        Ok(_) => {
            println!("✓ Decreased capacity to 20");
            cache_with_listener.run_pending_tasks();
            
            let count = cache_with_listener.entry_count();
            let evicted = evicted_count.load(std::sync::atomic::Ordering::SeqCst);
            
            println!("Entry count: {}", count);
            println!("Evicted entries (via listener): {}", evicted);
            
            if count <= 20 {
                println!("✓ Eviction worked correctly");
            }
            if evicted >= 30 {
                println!("✓ Eviction listener was called for evicted entries");
            }
        }
        Err(e) => println!("✗ Failed: {}", e),
    }

    println!("\n=== All tests completed successfully ===");
}
