use std::{
    collections::HashMap,
    thread::{self, JoinHandle}, sync::{Arc, atomic::AtomicUsize},
};

use atomic_counter::{RelaxedCounter, AtomicCounter};
use crossbeam_channel::{unbounded, Receiver, Sender};
use log::info;

use super::{collector::CollectorResult, file_info::FileInfo};

pub struct DeduplicatorResult {
    
}

struct PairDeduplicatorResult {}

pub fn deduplicate(receiver: Receiver<CollectorResult>) -> JoinHandle<()> {
    let (s, r): (Sender<Vec<FileInfo>>, Receiver<Vec<FileInfo>>) = unbounded();

    let worker = thread::spawn(move || {
        info!("Started new thread");
        match receiver.recv() {
            Ok(message) => {
                let mut children = Vec::new();
                
                let counter = fun_name(r, &mut children, message.buckets.len());

                message.buckets.into_iter()
                    .for_each(|dup_pair| {
                        s.send(dup_pair);
                        ();
                    });

                
                children.into_iter()
                .for_each(|t| {
                    info!("counter {},  waiting for t", counter.load(std::sync::atomic::Ordering::SeqCst));
                    t.join();
                    ();
                });
                
                DeduplicatorResult { };
            }
            Err(_) => todo!(),
        }

        DeduplicatorResult { };
    });

    return worker;
}

fn fun_name(r: Receiver<Vec<FileInfo>>, children: &mut Vec<JoinHandle<()>>, total: usize) -> Arc<AtomicUsize> {
    let val = Arc::new(AtomicUsize::new(5));
    for i in 0..3 {
        let atomic_counter_c = Arc::clone(&val);
        let receiver = r.clone();
        let worker = thread::spawn(move || {
            info!("Started new deduplicateur thread");
            loop {
                match receiver.recv() {
                    Ok(result) => {          
                        atomic_counter_c.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        info!("{} - Processing pair with len {}", i, result.len());
                    },
                    Err(_) => break,
                }
                if total == atomic_counter_c.load(std::sync::atomic::Ordering::SeqCst) {
                    info!("Not waiting for new messages");
                    break;
                }
            }
        });
        children.push(worker);
    };
    return val;
}
