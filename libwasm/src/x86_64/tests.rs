use super::*;

#[test]
fn return_value() {
    use testing::Emulator;
    let src = r#"
(module

    (func (export "foo") (result i64)
     i64.const 42
    )
)
"#;
    let binary = wat::parse_str(src).expect("binary module");
    let module = X86_64Compiler::default()
        .compile(&binary)
        .expect("compiled module");

    let mut emulator = Emulator::new().expect("emulator");
    let emu_mod = emulator.add_module(module).expect("module addition");
    emulator
        .call_function(emu_mod.clone(), "foo")
        .expect("call");

    assert_eq!(emulator.read_register(testing::RAX).unwrap(), 42);
}

#[test]
fn passing_args_and_return_value() {
    use testing::Emulator;
    let src = r#"
(module

    (func (export "foo") (param i64) (param i64) (result i64)
     i64.sub
    )
)
"#;
    let binary = wat::parse_str(src).expect("binary module");
    let module = X86_64Compiler::default()
        .compile(&binary)
        .expect("compiled module");

    let mut emulator = Emulator::new().expect("emulator");
    let emu_mod = emulator.add_module(module).expect("module addition");

    emulator.write_register(testing::RDI, 52).expect("1st arg");
    emulator.write_register(testing::RSI, 10).expect("2nd arg");

    emulator
        .call_function(emu_mod.clone(), "foo")
        .expect("call");

    assert_eq!(emulator.read_register(testing::RAX).unwrap(), 42);
}

#[test]
fn local_call() {
    use testing::Emulator;
    let src = r#"
(module

    (func (export "bar")
        call $foo
    )
    
    (func $foo
       call $foo1)
       
    (func $foo1)
    
    (func $unused)
)
"#;
    let binary = wat::parse_str(src).expect("binary module");
    let module = X86_64Compiler::default()
        .compile(&binary)
        .expect("compiled module");

    let mut emulator = Emulator::new().expect("emulator");
    let emu_mod = emulator.add_module(module).expect("module addition");
    emulator
        .call_function(emu_mod.clone(), "bar")
        .expect("call");

    assert_eq!(
        emu_mod
            .borrow()
            .instruction_execution_count(emu_mod.borrow().function_entry_point("bar").unwrap_or(0)),
        1
    );

    assert_eq!(
        emu_mod
            .borrow()
            .instruction_execution_count(emu_mod.borrow().function_entry_point(1).unwrap_or(0)),
        1
    );

    assert_eq!(
        emu_mod
            .borrow()
            .instruction_execution_count(emu_mod.borrow().function_entry_point(2).unwrap_or(0)),
        1
    );

    assert_eq!(
        emu_mod
            .borrow()
            .instruction_execution_count(emu_mod.borrow().function_entry_point(3).unwrap_or(0)),
        0
    );
}

#[test]
fn imported_wasm_call() {
    use testing::Emulator;
    let foo_src = r#"
(module

    (func $bar (import "b" "bar"))

    (func (export "foo")
        call $bar
    )
)
"#;
    let foo_binary = wat::parse_str(foo_src).expect("binary module");
    let foo_module = X86_64Compiler::default()
        .compile(&foo_binary)
        .expect("compiled module");

    let bar_src = r#"
(module
    (func (export "bar"))
)
"#;

    let bar_binary = wat::parse_str(bar_src).expect("binary module");
    let bar_module = X86_64Compiler::default()
        .compile(&bar_binary)
        .expect("compiled module");

    let mut emulator = Emulator::new().expect("emulator");
    let mod_foo = emulator.add_module(foo_module).expect("module addition");
    let mod_bar = emulator.add_module(bar_module).expect("module addition");

    let bar_module_offset = mod_bar.borrow().offset();
    let bar_function_offset =
        bar_module_offset + (mod_bar.borrow().function_entry_point("bar").unwrap() as u64);

    mod_foo
        .try_borrow_mut()
        .unwrap()
        .link_import("b", Some("bar"), bar_function_offset);

    emulator
        .call_function(mod_foo.clone(), "foo")
        .expect("call");

    assert_eq!(
        mod_foo
            .borrow()
            .instruction_execution_count(mod_foo.borrow().function_entry_point("foo").unwrap_or(0)),
        1
    );

    assert_eq!(
        mod_bar
            .borrow()
            .instruction_execution_count(mod_bar.borrow().function_entry_point("bar").unwrap_or(0)),
        1
    );
}

#[test]
fn external_call() {
    use testing::Emulator;
    let foo_src = r#"
(module

    (func $bar (import "b" "bar") (result i64))

    (func (export "foo") (result i64)
        call $bar
    )
)
"#;
    let foo_binary = wat::parse_str(foo_src).expect("binary module");
    let foo_module = X86_64Compiler::default()
        .compile(&foo_binary)
        .expect("compiled module");

    let mut emulator = Emulator::new().expect("emulator");
    let mod_foo = emulator.add_module(foo_module).expect("module addition");

    let mut assembler = CodeAssembler::new(64).expect("new assembler");
    use iced_x86::code_asm::*;
    assembler.push(rbp).expect("asm");
    assembler.mov(rbp, rsp).expect("asm");
    assembler.mov(eax, 42).expect("asm");
    assembler.pop(rbp).expect("asm");
    assembler.ret().expect("asm");
    let assembled = assembler.assemble(0).expect("asm");
    let bar_fun = emulator.add_memory(&assembled).expect("bar function");

    mod_foo
        .try_borrow_mut()
        .unwrap()
        .link_import("b", Some("bar"), bar_fun);

    emulator
        .call_function(mod_foo.clone(), "foo")
        .expect("call");

    assert_eq!(emulator.read_register(testing::RAX).unwrap(), 42);
}

#[test]
fn locals_basic() {
    use testing::Emulator;
    let foo_src = r#"
(module

    (func (export "foo") (result i64) (local i64) (local i64)
      i64.const 10
      local.set 0
      i64.const 32
      local.set 1
      local.get 0
      local.get 1
      i64.add
    )
)
"#;
    let foo_binary = wat::parse_str(foo_src).expect("binary module");
    let foo_module = X86_64Compiler::default()
        .compile(&foo_binary)
        .expect("compiled module");

    let mut emulator = Emulator::new().expect("emulator");
    let emu_mod = emulator.add_module(foo_module).expect("module addition");

    emulator
        .call_function(emu_mod.clone(), "foo")
        .expect("call");

    assert_eq!(emulator.read_register(testing::RAX).unwrap(), 42);
}
