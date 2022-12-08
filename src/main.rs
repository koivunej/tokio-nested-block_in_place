/*
use once_cell::sync::Lazy;

static FIRST: Lazy<tokio::runtime::Runtime> = Lazy::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
});

static SECOND: Lazy<tokio::runtime::Runtime> = Lazy::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
});
*/

fn main() {
    for attempt in 1.. {
        let first = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        let second = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        let (tx, mut rx) = tokio::sync::watch::channel(0usize);

        let _jh = first.spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(1)).await;

                tx.send_modify(|old| *old = old.wrapping_add(1));
            }
        });

        let jh = second.spawn(async move {
            tokio::task::block_in_place(|| {
                // println!("block_in_place");
                tokio::runtime::Handle::current().block_on(async {
                    // println!("{:indent$}-> block_on", "", indent = 2);

                    // this is required
                    tokio::task::block_in_place(|| {
                        // println!("{:indent$}-> block_in_place", "", indent = 4);
                    });

                    // println!("{:indent$}block_on continues", "", indent = 2);

                    {
                        let i = *rx.borrow();

                        loop {
                            rx.changed().await.unwrap();

                            if *rx.borrow() >= i + 2 {
                                break;
                            }
                        }
                    }

                    // not required, but was in the original code, seems to make this more
                    // reproducable
                    tokio::task::block_in_place(|| {
                        // dostuff
                        // println!("{:indent$}-> block_in_place", "", indent = 4);
                    });

                    // println!("{:indent$}block_on continues", "", indent = 2);
                });

                std::thread::sleep(std::time::Duration::from_micros(999));
                // println!(
                //     "{:indent$}after std::thread::sleep, before panic",
                //     "",
                //     indent = 2
                // );
            });
        });

        let res = second.block_on(jh);

        first.shutdown_timeout(std::time::Duration::from_secs(1));
        second.shutdown_timeout(std::time::Duration::from_secs(1));

        let e = match res {
            Ok(()) => continue,
            Err(e) => e,
        };

        let panic = e.into_panic();

        let s = *panic.downcast_ref::<&'static str>().unwrap();
        assert_eq!("assertion failed: cx_core.is_none()", s);

        println!();
        println!("hit assertion as expected: {s:?} in {attempt}th attempt");
        break;
    }
}
