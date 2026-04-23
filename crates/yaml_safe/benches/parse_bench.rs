use criterion::{
    black_box, criterion_group, criterion_main, Criterion,
};
use serde::Deserialize;
use yaml_safe::Value;

const SMALL_MAP: &str = "\
name: Alice
age: 30
active: true
";

const MEDIUM_MAP: &str = "\
database:
  host: localhost
  port: 5432
  name: mydb
  user: admin
  pool_size: 10
  ssl: true
logging:
  level: info
  format: json
  file: /var/log/app.log
  rotate: true
  max_size: 100
server:
  bind: 0.0.0.0
  port: 8080
  workers: 4
  timeout: 30
  keep_alive: true
";

fn large_sequence(n: usize) -> String {
    let mut s = String::new();
    for i in 0..n {
        s.push_str(&format!("- item_{}\n", i));
    }
    s
}

fn large_mapping(n: usize) -> String {
    let mut s = String::new();
    for i in 0..n {
        s.push_str(&format!("key_{}: value_{}\n", i, i));
    }
    s
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct SmallConfig {
    name: String,
    age: u32,
    active: bool,
}

fn bench_parse_small_map(c: &mut Criterion) {
    c.bench_function("parse_small_map_to_value", |b| {
        b.iter(|| {
            let _: Value =
                yaml_safe::from_str(black_box(SMALL_MAP)).unwrap();
        })
    });
    c.bench_function("parse_small_map_to_struct", |b| {
        b.iter(|| {
            let _: SmallConfig =
                yaml_safe::from_str(black_box(SMALL_MAP)).unwrap();
        })
    });
}

fn bench_parse_medium_map(c: &mut Criterion) {
    c.bench_function("parse_medium_map", |b| {
        b.iter(|| {
            let _: Value =
                yaml_safe::from_str(black_box(MEDIUM_MAP)).unwrap();
        })
    });
}

fn bench_parse_large_sequence(c: &mut Criterion) {
    let yaml = large_sequence(1000);
    c.bench_function("parse_seq_1000", |b| {
        b.iter(|| {
            let _: Value =
                yaml_safe::from_str(black_box(&yaml)).unwrap();
        })
    });
}

fn bench_parse_large_mapping(c: &mut Criterion) {
    let yaml = large_mapping(1000);
    c.bench_function("parse_map_1000", |b| {
        b.iter(|| {
            let _: Value =
                yaml_safe::from_str(black_box(&yaml)).unwrap();
        })
    });
}

fn bench_serialize(c: &mut Criterion) {
    let yaml = large_mapping(100);
    let v: Value = yaml_safe::from_str(&yaml).unwrap();
    c.bench_function("serialize_map_100", |b| {
        b.iter(|| {
            let _ = yaml_safe::to_string(black_box(&v)).unwrap();
        })
    });
}

fn bench_roundtrip(c: &mut Criterion) {
    c.bench_function("roundtrip_medium_map", |b| {
        b.iter(|| {
            let v: Value =
                yaml_safe::from_str(black_box(MEDIUM_MAP)).unwrap();
            let yaml = yaml_safe::to_string(&v).unwrap();
            let _: Value = yaml_safe::from_str(&yaml).unwrap();
        })
    });
}

criterion_group!(
    benches,
    bench_parse_small_map,
    bench_parse_medium_map,
    bench_parse_large_sequence,
    bench_parse_large_mapping,
    bench_serialize,
    bench_roundtrip,
);
criterion_main!(benches);
