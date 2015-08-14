extern crate hsm_gen;

fn main() {
    let generator = hsm_gen::HsmGenerator::new();
    generator.test_modification();
    generator.print();
}
