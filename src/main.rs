use crossbeam::{after, channel, never, select};
use std::time::Duration;
use std::{thread, time};

#[derive(Debug)]
struct Message {
    ping_pong_value: i32,
    stop_value: i32,
}

fn main() {
    let (tx1, rx1) = channel::unbounded::<Message>();
    let (tx2, rx2) = channel::unbounded::<Message>();
    let (tx_terminate, rx_terminate) = channel::unbounded::<String>();

    let msg = Message {
        ping_pong_value: 0,
        stop_value: 5,
    };

    // Change to a short duration for example for 1000 milliseconds or something to make
    // ping + pong threads exit after timing out. Otherwise it will exit after ping_pong_value reaches >= stop_value.
    let time_out = Some(Duration::from_millis(3000));

    // Create a channel that times out after the specified duration.
    // let timeout = duration.map(|d| after(d)).unwrap_or(never());
    let ping_timeout = time_out.map(after).unwrap_or_else(never);
    let pong_timeout = time_out.map(after).unwrap_or_else(never);

    let ping = || {
        let mut msg = msg;
        loop {
            select! {
                recv(ping_timeout) -> _ => {
                    println!("Ping: timed out");
                    break;
                },
                recv(rx_terminate) -> x => {
                    println!("{}", x.unwrap());
                    break;
                },
                default => {
                    if msg.ping_pong_value >= msg.stop_value {
                        tx_terminate.send("Ping complete: Terminate remote thread.".to_owned()).unwrap();
                        break;
                    }

                    println!("Ping: {:?}", msg);
                    tx1.send(msg).unwrap();
                    msg = rx2.recv().unwrap();
                    println!("Ping received: {:?}", msg);
                    thread::sleep(time::Duration::from_millis(500));
                },
            }
        }
    };

    crossbeam::scope(|s| {
        s.spawn(|_| ping());

        s.spawn(|_| loop {
            select! {
                recv(rx1) -> x => {
                                let mut msg:Message = x.unwrap();
                                println!("Pong received: {:?}", msg);
                                msg.ping_pong_value += 1;
                                tx2.send(msg).unwrap();
                              },
                recv(pong_timeout) -> _ => {
                    println!("Pong: timed out");
                    break;
                },
                recv(rx_terminate) -> x => {
                    println!("Pong: {}", x.unwrap());
                    break;
                },
            }
        });
    })
    .unwrap();
}
