#[allow(unused_macros)]
macro_rules! hVec {
    ($size:tt, $array:expr) => {{
        let mut t = heapless::Vec::<_, $size>::new();
        t.extend_from_slice(&$array).unwrap();
        t
    }};
}

#[allow(unused_macros)]
macro_rules! write_data {
    ($producer:ident, $data:expr) => {
        let mut wgr = $producer.grant_exact($data.len()).unwrap();
        wgr.buf().copy_from_slice(&$data);
        wgr.commit($data.len());
    };
}

#[allow(unused_macros)]
macro_rules! assert_bufs_eq {
    ($consumer:expr, $buf1:expr) => {
        assert_bufs_eq!($consumer, $buf1, &[]);
    };
    ($consumer:expr, $buf1:expr, $buf2:expr) => {
        let rgr = $consumer.split_read().unwrap();
        let (buf1, buf2) = rgr.bufs();
        assert_eq!(buf1, $buf1);
        assert_eq!(buf2, $buf2);
        rgr.release(0);
    };
}
