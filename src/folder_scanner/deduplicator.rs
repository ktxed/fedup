use std::{
    fs::{read, File},
    io::{Read, Seek, SeekFrom},
    thread::{self, JoinHandle},
};

use super::{collector::CollectorResult, file_info::FileInfo, duplicates_group::{Sample, HashedSample, DuplicatesGroup}};
use crossbeam_channel::{unbounded, Receiver, Sender};
use data_encoding::HEXLOWER;
use log::{debug, info, warn};
use ring::digest::{Context, SHA256};
use grouping_by::GroupingBy;

enum Message {
    Request { file_info: Vec<FileInfo> },
    Terminate,
}

const max_workers: usize = 3;

pub fn deduplicate(receiver: Receiver<CollectorResult>, result_publisher: &Sender<Option<DuplicatesGroup>>) -> JoinHandle<()> {
    let (s, r): (Sender<Message>, Receiver<Message>) = unbounded();
    let publisher = result_publisher.clone();
    let worker = thread::spawn(move || {
        info!("Started new thread");
        match receiver.recv() {
            Ok(message) => {
                let mut children = Vec::new();

                handle_message(r, &mut children, &publisher);

                message.buckets.into_iter().for_each(|dup_pair| {
                    s.send(Message::Request {
                        file_info: dup_pair,
                    })
                    .unwrap_or_default();
                    ();
                });

                // signalling worker threads to gracefully close
                for i in 0..max_workers {
                    s.send(Message::Terminate).unwrap_or_default();
                }

                children.into_iter().for_each(|t| {
                    t.join().unwrap_or_default();
                    ();
                });

                info!("Signal end to publisher");
                publisher.send(None);
              }
            Err(_) => todo!(),
        }
    });

    return worker;
}

fn handle_message(r: Receiver<Message>, children: &mut Vec<JoinHandle<()>>, result_publisher: &Sender<Option<DuplicatesGroup>>) -> () {
    for _ in 0..max_workers {
        let receiver = r.clone();
        let publisher = result_publisher.clone();
        let worker = thread::spawn(move || {
            let thread_id = thread::current().id();
            debug!("{:?} Started new deduplicateur thread", thread_id);
            loop {
                match receiver.recv() {
                    Ok(result) => match result {
                        Message::Request {
                            file_info: file_infos,
                        } => {
                            debug!(
                                "{:?} - Processing pair with len {}",
                                thread_id,
                                file_infos.len()
                            );

                            let samples = file_infos
                                .iter()
                                .filter_map(extract_sample)
                                .filter_map(hash_sample)
                                .collect::<Vec<HashedSample>>();
                              
                            // info!("{:?} - Sampled {} files", thread_id, samples.len());
                            
                            let grouped_samples = samples
                                .iter()
                                .grouping_by(|s| s.hash.clone());
                                                    
                            let duplicate_samples = grouped_samples.values()
                                .into_iter()
                                .filter(|value| value.len() > 1)
                                .collect::<Vec<&Vec<&HashedSample>>>();
                        
                            if (duplicate_samples.len() > 1) {
                                duplicate_samples.into_iter()
                                .for_each(|g| {
                                    let duplicates_group_vec = g.into_iter()
                                    .map(|x| (*x).clone())
                                    .collect::<Vec<HashedSample>>();
                                    let duplicates_group = DuplicatesGroup { item: duplicates_group_vec };
                                    info!("Found duplicate");
                                    publisher.send(Some(duplicates_group));
                                })

                            }
                        }
                        Message::Terminate => {
                            info!("{:?} - Quitting", thread_id);
                            break;
                        }
                    },
                    Err(_) => break,
                }
            }
        });
        children.push(worker);
    }
}

const PIECES: usize = 8;
const PIECE_SIZE_KIB: usize = 10;

fn extract_sample(file_info: &FileInfo) -> Option<Sample> {
    let thread_id = thread::current().id();
    let total_size = file_info.size;

    // if file size is larger than 512 KiB we sample 8 pieces of max 10 KiB
    // otherwise the sample is the file itself
    if (total_size <= 512 * 1024) {
        let sample = match read(file_info.file.clone()) {
            Ok(sample) => Option::Some(Sample {
                file_info: file_info.clone(),
                bytes: sample,
            }),
            Err(_) => Option::None,
        };
        return sample;
    } else {
        match File::open(&file_info.file) {
            Ok(mut handle) => {
                let mut sample_buffer = [0u8; PIECES * PIECE_SIZE_KIB * 1024];

                for i in 0..PIECES {
                    debug!("{:?} - Extracting sample {}", thread_id, i);

                    let buffer_slice = &mut sample_buffer
                        [i * PIECE_SIZE_KIB * 1024..(i + 1) * PIECE_SIZE_KIB * 1024];
                    handle
                        .seek(SeekFrom::Start(
                            (i * PIECE_SIZE_KIB * 1024).try_into().unwrap(),
                        ))
                        .unwrap_or(0);

                    match handle.read(buffer_slice) {
                        Ok(total_read) => debug!(
                            "{:?} - Read {} bytes from {} to buffer {}",
                            thread_id,
                            total_read,
                            file_info.file,
                            buffer_slice.len()
                        ),
                        Err(error) => warn!(
                            "{:?} - Failed reading from {}: {}",
                            thread_id, file_info.file, error
                        ),
                    }
                }
                return Option::Some(Sample {
                    file_info: file_info.clone(),
                    bytes: Vec::from(sample_buffer),
                });
            }
            Err(error) => {
                warn!(
                    "{:?} - Opening {} failed: {}",
                    thread_id, file_info.file, error
                );
                Option::None
            }
        }
    }
}

fn hash_sample(sample: Sample) -> Option<HashedSample> {
    let mut context = Context::new(&SHA256);
    context.update(&sample.bytes);
    let digest = context.finish();
    return Option::Some(HashedSample {
        sample: sample,
        hash: HEXLOWER.encode(digest.as_ref()),
    });
}
