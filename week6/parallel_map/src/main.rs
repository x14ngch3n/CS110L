use crossbeam_channel;
use std::{thread, time};

fn parallel_map<T, U, F>(input_vec: Vec<T>, num_threads: usize, f: F) -> Vec<U>
where
    F: FnOnce(T) -> U + Send + Copy + 'static,
    T: Send + 'static,
    U: Send + 'static + Default,
{
    let mut output_vec: Vec<U> = Vec::with_capacity(input_vec.len());
    output_vec.resize_with(input_vec.len(), Default::default);
    // create two channels for sending data between worker threads and main thread
    let (in_sender, in_receiver) = crossbeam_channel::unbounded();
    let (out_sender, out_receiver) = crossbeam_channel::unbounded();
    // spawn the worker threads
    let mut handles = Vec::new();
    for _ in 0..num_threads {
        let out_sender_clone = out_sender.clone();
        let in_receiver_clone = in_receiver.clone();
        handles.push(thread::spawn(move || {
            // wait for in_senders
            while let Ok((idx, num)) = in_receiver_clone.recv() {
                let res = f(num);
                // send back to out_receivers
                out_sender_clone
                    .send((idx, res))
                    .expect("There's no out_receiver");
            }
        }))
    }

    // send data to worker threads
    for (idx, num) in input_vec.into_iter().enumerate() {
        in_sender.send((idx, num)).expect("There's no in_receiver");
    }

    // tells in_receivers there's no in_senders, otherwise worker threads will hanging
    drop(in_sender);

    // tells out_receivers there's no out_senders, otherwise main thread will hanging
    drop(out_sender);

    // receive result in order
    while let Ok((idx, res)) = out_receiver.recv() {
        output_vec[idx] = res;
    }

    // join all worker threads
    for handle in handles {
        handle.join().unwrap();
    }

    output_vec
}

fn main() {
    let v = vec![6, 7, 8, 9, 10, 1, 2, 3, 4, 5, 12, 18, 11, 5, 20];
    let squares = parallel_map(v, 10, |num| {
        println!("{} squared is {}", num, num * num);
        thread::sleep(time::Duration::from_millis(500));
        num * num
    });
    println!("squares: {:?}", squares);
}
