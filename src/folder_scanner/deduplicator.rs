use std::{
    thread::{self, JoinHandle},
};

use crossbeam_channel::{unbounded, Receiver, Sender};
use log::info;

use crate::folder_scanner;

use super::{collector::CollectorResult, file_info::FileInfo};

enum Message {
    Request { 
        file_info: Vec<FileInfo>
    },
    Terminate
}

pub struct DeduplicatorResult {
    
}

struct PairDeduplicatorResult {}

pub fn deduplicate(receiver: Receiver<CollectorResult>) -> JoinHandle<()> {
    let (s, r): (Sender<Message>, Receiver<Message>) = unbounded();

    let worker = thread::spawn(move || {
        info!("Started new thread");
        match receiver.recv() {
            Ok(message) => {
                let mut children = Vec::new();
                
                fun_name(r, &mut children);

                message.buckets.into_iter()
                    .for_each(|dup_pair| {
                        s.send(Message::Request { file_info: dup_pair });
                        ();
                    });

                // signalling worker threads to gracefully close
                for i in 0..3 {
                    s.send(Message::Terminate);
                }

                
                children.into_iter()
                .for_each(|t| {
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

fn fun_name(r: Receiver<Message>, children: &mut Vec<JoinHandle<()>>) -> () {
    for i in 0..3 {
        let receiver = r.clone();
        let worker = thread::spawn(move || {
            let thread_id = thread::current().id();
            info!("{:?} Started new deduplicateur thread", thread_id);
            loop {
                match receiver.recv() {
                    Ok(result) => {
                        match result {          
                            Message::Request { file_info } => {
                                info!("{:?} - Processing pair with len {}", thread_id, file_info.len());
                            },
                            Message::Terminate => {
                                info!("{:?} Quitting", thread_id);
                                break
                            },
                        }
                    },
                    Err(_) => break,
                }
            }
        });
        children.push(worker);
    };
}
