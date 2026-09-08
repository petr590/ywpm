use criterion::{Bencher, Criterion, criterion_group, criterion_main};
use rand::RngExt;
use std::error::Error;
use std::io::{BufReader, BufWriter, Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::thread;

use ywpm::{reader, writer};

macro_rules! bench_wrapper {
    ($client_func:expr, $server_func:expr, $vec:expr) => {
        |bencher: &mut Bencher<'_>| {
            bencher.iter(|| {
                let _ = std::fs::remove_file(SOCKET_PATH);
                let listener = UnixListener::bind(SOCKET_PATH).unwrap();

                let server_handle = thread::spawn(move || {
                    if let Ok((mut server_stream, _)) = listener.accept() {
                        $server_func(&mut server_stream).unwrap();
                    }
                });

                let mut client_stream = UnixStream::connect(SOCKET_PATH).unwrap();
                $client_func(&mut client_stream, $vec).unwrap();

                server_handle.join().unwrap();
            });
        }
    };
}

const SOCKET_PATH: &str = "/tmp/ywpm.socket";

fn client_v1(stream: &mut UnixStream, vec: &Vec<String>) -> Result<(), Box<dyn Error>> {
    writer::write_string_vec(stream, vec)?;
    Ok(())
}

fn server_v1(stream: &mut UnixStream) -> Result<(), Box<dyn Error>> {
    reader::read_string_vec(stream)?;
    Ok(())
}

fn client_v2(stream: &mut UnixStream, vec: &Vec<String>) -> Result<(), Box<dyn Error>> {
    writer::write_string_vec(&mut BufWriter::new(stream), vec)?;
    Ok(())
}

fn server_v2(stream: &mut UnixStream) -> Result<(), Box<dyn Error>> {
    reader::read_string_vec(&mut BufReader::new(stream))?;
    Ok(())
}

fn client_v3(stream: &mut UnixStream, vec: &Vec<String>) -> Result<(), Box<dyn Error>> {
    let sum = vec.iter().map(|s| s.len()).sum::<usize>();
    let size = 4 + 4 * vec.len() + sum;

    let mut buffer = vec![0u8; size + 4];
    buffer[..4].copy_from_slice(&(size as u32).to_be_bytes());

    let mut slice = &mut buffer[4..];
    writer::write_string_vec(&mut slice, vec)?;

    stream.write_all(&buffer)?;
    Ok(())
}

fn server_v3(stream: &mut UnixStream) -> Result<(), Box<dyn Error>> {
    let mut header = [0u8; 4];
    stream.read_exact(&mut header)?;

    let size = u32::from_be_bytes(header) as usize;
    let mut buffer = vec![0u8; size];
    stream.read_exact(&mut buffer)?;

    let mut slice = &buffer[..];
    reader::read_string_vec(&mut slice)?;
    Ok(())
}

fn get_vectors() -> (Vec<String>, Vec<String>, Vec<String>) {
    let small_vec = vec![String::from("ywpm"), String::from("random")];

    let medium_vec = vec![
        String::from("ywpm"),
        String::from("add-to-group"),
        String::from("nier"),
        String::from("/home/user/some/path/to/nier/wallpapers/123456.jpeg"),
        String::from("--mode"),
        String::from("cover top hcenter"),
    ];

    let mut rng = rand::rng();
    let mut huge_str = String::new();
    huge_str.reserve_exact(1_000);

    for _ in 0..1_000 {
        huge_str.push(rng.random_range(32..=126) as u8 as char);
    }

    let mut huge_vec = Vec::new();
    huge_vec.reserve_exact(100_000);

    for _ in 0..100_000 {
        huge_vec.push(huge_str.clone());
    }

    (small_vec, medium_vec, huge_vec)
}

fn bench_socket_read_write(criterion: &mut Criterion) {
    let (small_vec, medium_vec, huge_vec) = get_vectors();

    let mut group = criterion.benchmark_group("UnixStream Performance");

    group.sample_size(50);

    group.bench_function(
        "Format V1 (Raw UnixStream, small vec)",
        bench_wrapper!(client_v1, server_v1, &small_vec),
    );
    group.bench_function(
        "Format V1 (Raw UnixStream, medium vec)",
        bench_wrapper!(client_v1, server_v1, &medium_vec),
    );
    group.bench_function(
        "Format V1 (Raw UnixStream, huge vec)",
        bench_wrapper!(client_v1, server_v1, &huge_vec),
    );

    group.bench_function(
        "Format V2 (Buf, small vec)",
        bench_wrapper!(client_v2, server_v2, &small_vec),
    );
    group.bench_function(
        "Format V2 (Buf, medium vec)",
        bench_wrapper!(client_v2, server_v2, &medium_vec),
    );
    group.bench_function(
        "Format V2 (Buf, huge vec)",
        bench_wrapper!(client_v2, server_v2, &huge_vec),
    );

    group.bench_function(
        "Format V3 (Buf, small vec)",
        bench_wrapper!(client_v3, server_v3, &small_vec),
    );
    group.bench_function(
        "Format V3 (Buf, medium vec)",
        bench_wrapper!(client_v3, server_v3, &medium_vec),
    );
    group.bench_function(
        "Format V3 (Buf, huge vec)",
        bench_wrapper!(client_v3, server_v3, &huge_vec),
    );

    group.finish();

    let _ = std::fs::remove_file(SOCKET_PATH);
}

criterion_group!(benches, bench_socket_read_write);
criterion_main!(benches);
