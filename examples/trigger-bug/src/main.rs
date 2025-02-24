use std::fs;
use polkavm::{Caller, Config, Engine, Linker, Module, ProgramBlob};

fn main() {
    env_logger::init();

    // compile
    let input_path = std::env::var("INPUT").expect("no INPUT in env");

    // link
    let mut config = polkavm_linker::Config::default();
    config.set_optimize(false);

    let orig = fs::read(input_path).expect("Failed to read {input_path:?}");
    let linked = polkavm_linker::program_from_elf(config, orig.as_ref())
        .unwrap();

    let blob = ProgramBlob::parse(linked[..].into()).unwrap();

    let config = Config::from_env().unwrap();
    let engine = Engine::new(&config).unwrap();
    let module = Module::from_blob(&engine, &Default::default(), blob).unwrap();

    // High-level API.
    let mut linker: Linker = Linker::new();

    linker.define_typed(
        "value_transferred",
        |caller: Caller<()>, out_ptr: u32| {
            let data = 0u128.to_le_bytes();
            caller.instance.write_memory(out_ptr, &data[..]).unwrap();
            println!("wrote value to program memory");
        },
    ).unwrap();

    // Link the host functions with the module.
    let instance_pre = linker.instantiate_pre(&module).unwrap();

    // Instantiate the module.
    let mut instance = instance_pre.instantiate().unwrap();

    // Grab the function and call it.
    println!("Calling into the guest program:");
    let _result = instance
        .call_typed_and_get_result::<(), ()>(&mut (), "deploy", ())
        .unwrap();
}
