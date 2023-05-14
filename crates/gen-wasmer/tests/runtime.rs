use anyhow::Result;
use wasmer::{Imports, Instance, Module, Store};
use wasmer_wasi::WasiState;

test_helpers::runtime_tests_wasmer!();

pub fn instantiate<T, I>(
    wasm: &str,
    store: &mut Store,
    add_imports: impl FnOnce(&mut Store, &mut Imports) -> I,
    mk_exports: impl FnOnce(&mut Store, &Module, &mut Imports) -> Result<(T, Instance)>,
) -> Result<T>
where
    I: FnOnce(&Instance, &dyn wasmer::AsStoreRef) -> Result<(), anyhow::Error>,
{
    // let module = Module::from_file(&*store, wasm)?;
    // let bytes = std::fs::read(wasm)?;
    // panic!("WHAT IS IT? {:?}", wasm);

    // let mut bytes = Vec::<u8>::new();
    // let mut escaped = false;
    // let mut iter = wasm.bytes();
    // while let Some(b) = iter.next() {
    //     if escaped {
    //         match b {
    //             b'x' => {
    //                 let mut hex = String::new();
    //                 hex.push(iter.next().unwrap() as char);
    //                 hex.push(iter.next().unwrap() as char);
    //                 let byte = u8::from_str_radix(&hex, 16).unwrap();
    //                 bytes.push(byte);
    //             }
    //             _ => panic!("Invalid escape sequence: {:?}", b),
    //         }
    //         escaped = false;
    //     } else if b == b'\\' {
    //         escaped = true;
    //     } else {
    //         bytes.push(b);
    //     }
    // }

    let mut bytes = Vec::<u8>::with_capacity(wasm.len() / 2);
    for index in (0..wasm.len()/2).map(|x| x * 2) {
        bytes.push(u8::from_str_radix(&wasm[index..index+2], 16).unwrap());
    }

    let module = Module::from_binary(&*store, &bytes)?;

    let wasi_env = WasiState::new("test").finalize(store)?;
    let mut imports = wasi_env
        .import_object(store, &module)
        .unwrap_or(Imports::new());

    let initializer = add_imports(store, &mut imports);

    let (exports, instance) = mk_exports(store, &module, &mut imports)?;

    let memory = instance.exports.get_memory("memory")?;
    wasi_env.data_mut(store).set_memory(memory.clone());

    initializer(&instance, store)?;

    Ok(exports)
}

// crates/gen-wasmer$ wasm-pack test --node --features js
#[cfg_attr(feature = "js", wasm_bindgen_test::wasm_bindgen_test)]
fn should_fail() {
    // panic!()
    numbers_rust::test().unwrap()
}
