use std::collections::HashMap;

use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::test]
async fn test_signal() {
    use moonbase::signal::*;

    let sig = Signal::new();
    let sender = sig.get_sender();
    fn spawn_sig_recv(sig: &Signal) -> tokio::task::JoinHandle<()> {
        let sig = sig.clone();
        tokio::spawn(async move {
            let mut count = 0;
            loop {
                sig.recv().await;
                let msg = format!("signal recv {}\n", count);
                tokio::io::stdout()
                    .write_all(msg.as_bytes())
                    .await
                    .expect("write fail");
                count += 1;
            }
        })
    }


    let mut rb = Vec::with_capacity(1024);
    let mut handle_repo = HashMap::new();
    loop {
        tokio::io::stdin()
            .read_buf(&mut rb)
            .await
            .expect("read fail");
        if rb.starts_with(b"quit\n") {
            break;
        } else if let Some(q) = rb.strip_prefix(b"spawn ") {
            let h = spawn_sig_recv(&sig);
            handle_repo.insert(q.to_vec(), h);
        } else if let Some(q) = rb.strip_prefix(b"abort ") {
            if let Some(h) = handle_repo.remove(q) {
                h.abort();
            }
        } else {
            sender.send();
        }

        rb.clear();
    }
}
