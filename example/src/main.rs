use colfer::Message;

mod bench_colfer;
mod bench_pb;
mod gen;

use prost::bytes::Bytes;
use std::io::Cursor;
use std::time::Instant;

fn bench_colfer() {
    use bench_colfer::Colfer;

    let test_data = vec![
        Colfer {
            key: 1234567890,
            host: "db003lz12".to_string(),
            port: 389,
            size: 452,
            hash: 0x488b5c2428488918,
            ratio: 0.99,
            route: true,
        },
        Colfer {
            key: 1234567891,
            host: "localhost".to_string(),
            port: 22,
            size: 4096,
            hash: 0x243048899c24c824,
            ratio: 0.20,
            route: false,
        },
        Colfer {
            key: 1234567892,
            host: "kdc.local".to_string(),
            port: 88,
            size: 1984,
            hash: 0x000048891c24485c,
            ratio: 0.06,
            route: false,
        },
        Colfer {
            key: 1234567893,
            host: "vhost8.dmz.example.com".to_string(),
            port: 27017,
            size: 59741,
            hash: 0x5c2408488b9c2489,
            ratio: 0.0,
            route: true,
        },
    ];

    let s = Instant::now();
    let count = 1000000;
    let mut data = Vec::new();

    for _ in 0..count {
        for c in &test_data {
            c.encode(&mut data);
        }
    }
    println!(
        "COLFER encode: {:.03}s size: {}",
        (Instant::now() - s).as_secs_f32(),
        data.len()
    );

    let s = Instant::now();
    let mut r = Cursor::new(data);
    for _ in 0..count {
        for _ in 0..4 {
            Colfer::decode(&mut r);
        }
    }
    println!("COLFER decode: {:.03}s", (Instant::now() - s).as_secs_f32());
}

fn bench_pb() {
    use bench_pb::Colfer;
    use prost::Message;

    let test_data = vec![
        Colfer {
            key: 1234567890,
            host: "db003lz12".to_string(),
            port: 389,
            size: 452,
            hash: 0x488b5c2428488918,
            ratio: 0.99,
            route: true,
        },
        Colfer {
            key: 1234567891,
            host: "localhost".to_string(),
            port: 22,
            size: 4096,
            hash: 0x243048899c24c824,
            ratio: 0.20,
            route: false,
        },
        Colfer {
            key: 1234567892,
            host: "kdc.local".to_string(),
            port: 88,
            size: 1984,
            hash: 0x000048891c24485c,
            ratio: 0.06,
            route: false,
        },
        Colfer {
            key: 1234567893,
            host: "vhost8.dmz.example.com".to_string(),
            port: 27017,
            size: 59741,
            hash: 0x5c2408488b9c2489,
            ratio: 0.0,
            route: true,
        },
    ];

    let s = Instant::now();
    let count = 1000000;
    let mut data: Vec<u8> = Vec::new();

    for _ in 0..count {
        for c in &test_data {
            c.encode(&mut data).unwrap();
        }
    }
    println!(
        "Protobuf encode: {:.03}s size: {}",
        (Instant::now() - s).as_secs_f32(),
        data.len()
    );

    let s = Instant::now();
    let mut buf = Bytes::from(data);
    for _ in 0..count {
        for _ in 0..4 {
            Colfer::decode(&mut buf).unwrap();
        }
    }
    println!(
        "Protobuf decode: {:.03}s",
        (Instant::now() - s).as_secs_f32()
    );
}

fn main() {
    bench_colfer();
    bench_pb();
}
