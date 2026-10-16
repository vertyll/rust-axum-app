//! Compiles the Tailwind stylesheet with the standalone Tailwind CLI, pinned
//! to one release and checked against that release's published checksums.

use topcoat::tailwind::BuildConfig;

const TAILWIND_VERSION: &str = "4.3.2";

fn main() {
    BuildConfig::new()
        .version_checksum(TAILWIND_VERSION, tailwind_checksum())
        .input("styles.css")
        .render()
        .expect("the Tailwind stylesheet compiles");
}

/// The SHA-256 of the Tailwind CLI for the platform the build runs on, from
/// the release's `sha256sums.txt`.
fn tailwind_checksum() -> &'static str {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "aarch64") => "sha256:b800b0659dc64b9f03ede5660244d9415d777d5739ae2889280877ca37be742a",
        ("macos", "x86_64") => "sha256:cef8f110471e889c3c4409055cf8aff33076f58a081867b0dfc6534b290bfbb0",
        ("linux", "aarch64") => "sha256:394ddccc2402cfa3abd97dfba56f3587781a3d6e6ce66e65ceada14beb7664b8",
        ("linux", "x86_64") => "sha256:5036c4fb4328e0bcdbb6065c70d8ac9452e0d4c947113a788a8f94fd390425c1",
        (os, arch) => panic!("no pinned Tailwind CLI checksum for {os}/{arch}"),
    }
}
