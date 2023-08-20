// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

fn main() {
    uniffi::generate_scaffolding("./src/keechain.udl").expect("Building the UDL file failed");
}
