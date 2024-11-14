pub mod symbols;

#[cfg(test)]
mod tests {
    use crate::symbols::SymbolLoader;

    #[test]
    fn test_get_kernel_symbol_address() {
        let loader = SymbolLoader::new(None).unwrap();
        let address = loader
            .get_kernel_symbol_address("KeUserModeCallback")
            .unwrap();

        println!("0x{:X}", address);
    }
}
