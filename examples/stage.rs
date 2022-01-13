use std::{sync::Arc, thread};

use dos_actors::{io, Actor, Initiator, Terminator};

fn main() -> anyhow::Result<()> {
    const N: usize = 1;
    type U = Vec<f64>;

    let time_idx = Arc::new(0usize);

    let (tx0, rx0) = flume::bounded::<io::S<U, N>>(10);
    let (tx1, rx1) = flume::bounded::<io::S<U, N>>(10);

    let u = vec![1.2345f64];

    let mut a2 = Terminator::<U, N>::new(
        time_idx.clone(),
        vec![io::Input::<U, N>::new(Vec::new(), rx1)],
    );

    let a0_time_idx = time_idx.clone();
    let a1_time_idx = time_idx.clone();
    let jhandle = vec![
        thread::spawn(move || {
            let a0 =
                Initiator::<U, N>::new(a0_time_idx, vec![io::Output::<U, N>::new(u, vec![tx0])]);
            a0.distribute();
        }),
        thread::spawn(move || {
            let mut a1 = Actor::<U, U, N, N>::new(
                a1_time_idx,
                vec![io::Input::<U, N>::new(Vec::new(), rx0)],
                vec![io::Output::<U, N>::new(Vec::new(), vec![tx1])],
            );
            a1.collect();
            let u: Vec<f64> = a1.inputs.as_ref().unwrap().get(0).unwrap().into();
            *Arc::get_mut(&mut a1.outputs.as_mut().unwrap().get_mut(0).unwrap().data).unwrap() =
                u.into();
            a1.distribute();
        }),
        thread::spawn(move || {
            a2.collect();
            dbg!(&a2);
        }),
    ];

    for jh in jhandle {
        jh.join();
    }

    Ok(())
}
