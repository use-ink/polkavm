use std::fs;
use polkavm::{Caller, Config, Engine, Error, InterruptKind, Linker, Module, ProgramBlob, Reg};

fn main() {
    env_logger::init();

    // compile
    let input_path = std::env::var("INPUT").expect("no INPUT in env");

    // link
    let mut config = polkavm_linker::Config::default();

    // the bug occurs, no matter if `true` set or not set here.
    // but it has a difference on which artifact is wrongfully output.
    config.set_optimize(true);

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
        "debug_message",
        |caller: Caller<()>, buffer: u32, length: u32| {
            let buffer = caller.instance.read_memory(buffer, length).unwrap();
            eprintln!("print: {:?}", String::from_utf8(buffer).unwrap());
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
