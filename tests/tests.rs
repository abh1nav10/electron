#[cfg(test)]
mod queue_test {
    use electron::Stack;
    #[test]
    fn test_one() {
        let new = &Stack::new();
        std::thread::scope(|s| {
            for i in 0..500 {
                s.spawn(move || {
                    let _ = new.insert(i);
                });
            }
        });
        std::thread::scope(|s| {
            for _ in 0..500 {
                s.spawn(move || {
                    let _ = new.delete();
                });
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use electron::Queue;
    #[test]
    fn test() {
        let queue = &Queue::new();
        std::thread::scope(|s| {
            for i in 0..20 {
                s.spawn(move || {
                    if i & 2 == 0 {
                        queue.enqueue(i);
                    } else {
                        let ret = queue.dequeue();
                        if let Ok(t) = ret {
                            println!("{:?}", t);
                        }
                    }
                });
            }
        });
    }
}
