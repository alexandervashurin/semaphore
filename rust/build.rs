//! Build script для gRPC
//! В этой версии мы используем предварительно сгенерированный код
//! чтобы избежать зависимости от protoc

fn main() {
    println!("cargo:rerun-if-changed=proto/semaphore.proto");
    println!("cargo:warning=Using pre-generated gRPC code to avoid protoc dependency");
}
