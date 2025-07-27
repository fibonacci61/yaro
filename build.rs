fn main() {
    println!("cargo::rerun-if-changed=yaro.ld");
    println!("cargo::rustc-link-arg=yaro.ld");
}
