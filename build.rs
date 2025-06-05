fn main() {
    #[cfg(feature = "embedded")]
    embuild::espidf::sysenv::output();
}
