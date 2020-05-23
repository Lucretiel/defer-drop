# defer-drop

A utility type that allows you to defer dropping your data to a background thread. Inspired by [this article](https://abramov.io/rust-dropping-things-in-another-thread) by Aaron Abramov.

## At a glance

```rust
// examples/demo.rs

println!("Allocating a ridiculously large vector");
let vec1: Vec<Vec<String>> = repeat_with(|| {
    repeat_with(|| "Hello, World".to_string())
        .take(1000)
        .collect()
})
.take(1000)
.collect();

println!("Duplicating that vector");
let vec2 = vec1.clone();

// Drop vec1 in a background thread
let defer_vec1 = DeferDrop::new(vec1);

println!("Dropping the vectors");

let vec1_timer = timer(move || drop(defer_vec1));
let vec2_timer = timer(move || drop(vec2));

println!("Duration of deferred drop: {:?}", vec1_timer);
println!("Duration of foreground drop: {:?}", vec2_timer);
```

On my machine, this prints:

```
Allocating a ridiculously large vector
Duplicating that vector
Dropping the vectors
Duration of deferred drop: 178µs
Duration of foreground drop: 102.5139ms
```

You can run this example yourself with:

```
cargo run --example demo
```

## Summary

defer-drop provides a wrapper type that, when dropped, sends the inner value to a global background thread to be dropped. Useful in cases where a value takes a long time to drop (for instance, a windows file that might block on close, or a large data structure that has to extensively recursively trawl itself).

## Notes

Carefully consider whether this pattern is necessary for your use case. Like all worker-thread abstractions, sending the value to a separate thread comes with its own costs, so it should only be done if performance profiling indicates that it's a performance gain.

There is only one global worker thread. Dropped values are enqueued in an unbounded channel to be consumed by this thread; if you produce more garbage than the thread can handle, this will cause unbounded memory consumption. There is currently no way for the thread to signal or block if it is overwhelmed.

All of the standard non-determinism threading caveats apply here. The objects are guaranteed to be destructed in the order received through a channel, which means that objects sent from a single thread will be destructed in order. However, there is no guarantee about the ordering of interleaved values from different threads. Additionally, there are no guarantees about how long the values will be queued before being dropped, or even that they will be dropped at all. If your `main` thread terminates before all drops could be completed, they will be silently lost (as though via a `mem::forget`.This behavior is entirely up to your OS's thread scheduler. There is no way to receive a signal indicating when a particular object was dropped.
