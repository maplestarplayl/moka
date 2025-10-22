use moka::sync::Cache;

fn main() {
    println!("=== Testing set_max_capacity ===\n");

    // Create a cache with initial capacity of 100
    let cache: Cache<String, String> = Cache::new(100);
    println!("Initial max capacity: {:?}", cache.policy().max_capacity());

    // Insert 50 entries
    println!("\nInserting 50 entries...");
    for i in 0..50 {
        cache.insert(format!("key-{}", i), format!("value-{}", i));
    }
    cache.run_pending_tasks();
    println!("Entry count: {}", cache.entry_count());

    // Increase capacity to 200
    println!("\n--- Increasing capacity to 200 ---");
    match cache.set_max_capacity(200) {
        Ok(_) => {
            println!("✓ Successfully increased capacity");
            println!("New max capacity: {:?}", cache.policy().max_capacity());
            println!("Entry count: {}", cache.entry_count());
        }
        Err(e) => println!("✗ Failed to increase capacity: {}", e),
    }

    // Decrease capacity to 30 - this should trigger eviction
    println!("\n--- Decreasing capacity to 30 ---");
    match cache.set_max_capacity(30) {
        Ok(_) => {
            println!("✓ Successfully decreased capacity");
            println!("New max capacity: {:?}", cache.policy().max_capacity());
            
            // Run pending tasks to ensure eviction completes
            cache.run_pending_tasks();
            
            let entry_count = cache.entry_count();
            println!("Entry count after eviction: {}", entry_count);
            
            if entry_count <= 30 {
                println!("✓ Eviction worked correctly (count: {} <= 30)", entry_count);
            } else {
                println!("✗ Eviction may need more time (count: {} > 30)", entry_count);
                println!("  Running pending tasks again...");
                cache.run_pending_tasks();
                println!("  Entry count: {}", cache.entry_count());
            }
        }
        Err(e) => println!("✗ Failed to decrease capacity: {}", e),
    }

    // Test setting capacity to zero
    println!("\n--- Setting capacity to 0 (disabled cache) ---");
    match cache.set_max_capacity(0) {
        Ok(_) => {
            println!("✓ Successfully set capacity to 0");
            println!("New max capacity: {:?}", cache.policy().max_capacity());
            cache.run_pending_tasks();
            println!("Entry count: {}", cache.entry_count());
        }
        Err(e) => println!("✗ Failed to set capacity to 0: {}", e),
    }

    // Verify that new insertions don't work with capacity 0
    println!("\n--- Testing insertion with capacity 0 ---");
    cache.insert("test-key".to_string(), "test-value".to_string());
    cache.run_pending_tasks();
    if cache.contains_key("test-key") {
        println!("✗ Entry was inserted (unexpected)");
    } else {
        println!("✓ Entry was not inserted (as expected with capacity 0)");
    }

    println!("\n=== Test completed ===");
}
