# Get windows kernel using windows DbgHelp API

## Start

~~~shell
cargo add moon-windows-symbol
~~~

### Dependences
symsrv.dll and dbghelp.dll

### Simple exampel
~~~rust
pub fn main(){
    let loader = SymbolLoader::new(None).unwrap();
    let address = loader
        .get_kernel_symbol_address("KeUserModeCallback")
        .unwrap();

    println!("0x{:X}", address);
}
~~~

### Assign pdb download path
~~~rust
pub fn main(){
    let loader = SymbolLoader::new(Some("C:\\Symbols")).unwrap();
    let address = loader
        .get_kernel_symbol_address("KeUserModeCallback")
        .unwrap();

    println!("0x{:X}", address);
}
~~~