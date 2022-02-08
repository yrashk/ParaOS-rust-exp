use crate::Compiler;
use alloc::borrow::ToOwned;
use alloc::collections::{BTreeMap, VecDeque};
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use byteorder::{ByteOrder, LittleEndian};
use core::mem::size_of;
use core::ops::{Deref, DerefMut};
use iced_x86::code_asm::{r11, r8, r9, rax, rcx, rdi, rdx, rsi, AsmRegister64, CodeAssembler};
use iced_x86::IcedError;
use wasmparser_nostd::*;

mod instructions;

#[derive(Debug)]
pub enum Error {
    WasmReaderError(BinaryReaderError),
    AssemblerError(IcedError),
}

impl From<BinaryReaderError> for Error {
    fn from(e: BinaryReaderError) -> Self {
        Self::WasmReaderError(e)
    }
}

impl From<IcedError> for Error {
    fn from(e: IcedError) -> Self {
        Self::AssemblerError(e)
    }
}

pub struct X86_64Compiler;

impl core::default::Default for X86_64Compiler {
    fn default() -> Self {
        X86_64Compiler
    }
}

pub struct Module {
    functions: BTreeMap<u32, usize>,
    function_bodies: BTreeMap<u32, usize>,
    exports: BTreeMap<String, u32>,
    imports: BTreeMap<u32, (String, Option<String>, usize)>,
}

pub struct FunctionIndex(u32);

pub trait FunctionIdentifier {
    fn find_function(&self, module: &Module) -> Option<u32>;
}

impl FunctionIdentifier for u32 {
    fn find_function(&self, module: &Module) -> Option<u32> {
        module.function_bodies.get(self).map(|_| *self)
    }
}

impl FunctionIdentifier for &str {
    fn find_function(&self, module: &Module) -> Option<u32> {
        module
            .exports
            .get(self as &str)
            .and_then(|index| (*index).find_function(module))
    }
}

impl Module {
    fn new() -> Self {
        Self {
            functions: BTreeMap::new(),
            function_bodies: BTreeMap::new(),
            exports: BTreeMap::new(),
            imports: BTreeMap::new(),
        }
    }

    fn assembled(self, assembled: Vec<u8>) -> AssembledModule {
        AssembledModule {
            module: self,
            assembled,
        }
    }

    pub fn function_entry_point<I: FunctionIdentifier>(&self, identifier: I) -> Option<usize> {
        identifier
            .find_function(self)
            .and_then(|idx| self.function_bodies.get(&idx).cloned())
    }
}

pub struct AssembledModule {
    module: Module,
    assembled: Vec<u8>,
}

impl Deref for AssembledModule {
    type Target = Module;

    fn deref(&self) -> &Self::Target {
        &self.module
    }
}

impl DerefMut for AssembledModule {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.module
    }
}

impl AssembledModule {
    pub fn binary(&self) -> &[u8] {
        &self.assembled
    }

    pub fn link_import(&mut self, module: &str, name: Option<&str>, addr: u64) {
        let relocation = self
            .imports
            .iter()
            .find_map(|(_, (module_, name_, offset))| {
                let names_equal = match (name, name_) {
                    (None, None) => false,
                    (None, Some(_)) => false,
                    (Some(_), None) => false,
                    (Some(name), Some(name_)) => name == name_,
                };
                if module_ == module && names_equal {
                    Some(*offset)
                } else {
                    None
                }
            });
        match relocation {
            Some(offset) => {
                let mut mem = &mut self.assembled[offset..offset + size_of::<u64>()];
                LittleEndian::write_u64(&mut mem, addr);
            }
            None => (),
        }
    }
}

impl Compiler for X86_64Compiler {
    type Error = Error;
    type Module = AssembledModule;

    fn compile(&self, module: &[u8]) -> Result<Self::Module, Self::Error> {
        let mut assembler = CodeAssembler::new(64)?;
        let mut got = BTreeMap::new();
        let mut ils = BTreeMap::new();
        let mut parser = wasmparser_nostd::Parser::new(0);
        let mut data: &[u8] = &module;
        let mut eof = false;
        let mut module = Module::new();
        let mut function_index = 0;
        let mut function_body_index = 0;
        let mut function_typedefs = BTreeMap::new();
        let mut function_type_index = 0;
        let mut function_types = BTreeMap::new();
        loop {
            let parsed = parser.parse(&data, eof)?;

            match parsed {
                Chunk::Parsed { payload, consumed } => {
                    match payload {
                        Payload::End => break,
                        Payload::TypeSection(ts) => {
                            for t in ts {
                                let typedef = t?;
                                match typedef {
                                    TypeDef::Func(func_type) => {
                                        function_typedefs.insert(function_type_index, func_type);
                                        function_type_index += 1;
                                    }
                                    _ => {}
                                }
                            }
                        }
                        Payload::ImportSection(is) => {
                            for i in is {
                                let import = i?;
                                let offset = assembler.assemble(0)?.len();
                                let reference = (
                                    import.module.to_owned(),
                                    import.field.map(str::to_owned),
                                    offset,
                                );
                                match import.ty {
                                    ImportSectionEntryType::Function(function_type) => {
                                        module.imports.insert(function_index, reference);
                                        let mut label = assembler.create_label();
                                        assembler.set_label(&mut label)?;
                                        assembler.dq(&[0xBADC0FFEE0DDF00D])?;
                                        ils.insert(function_index, label);
                                        function_types.insert(function_index, function_type);
                                        function_index += 1;
                                        function_body_index += 1;
                                    }
                                    _ => (),
                                }
                            }
                        }
                        Payload::FunctionSection(fs) => {
                            for function_type in fs.into_iter() {
                                let mut label = assembler.create_label();
                                assembler.set_label(&mut label)?;
                                let offset = assembler.instructions().len();
                                assembler.dq(&[0])?;
                                module.functions.insert(function_index, offset);
                                got.insert(function_index, assembler.create_label());
                                function_types.insert(function_index, function_type?);
                                function_index += 1;
                            }
                        }
                        Payload::ExportSection(es) => {
                            for e in es.into_iter() {
                                let export = e?;
                                module
                                    .exports
                                    .insert(String::from(export.field), export.index);
                            }
                        }
                        Payload::CodeSectionEntry(cs) => {
                            let function_type = function_types
                                .get(&function_body_index)
                                .and_then(|i| function_types.get(&i))
                                .and_then(|t| function_typedefs.get(t))
                                .cloned()
                                .unwrap();
                            let offset = assembler.assemble(0)?.len();
                            module.function_bodies.insert(function_body_index, offset);
                            let fun_label = got.get_mut(&function_body_index).unwrap();
                            assembler.set_label(fun_label)?;
                            let rd = cs.get_operators_reader()?;
                            assembler.pop(r11)?;
                            let mut integer_order: VecDeque<AsmRegister64> =
                                vec![rdi, rsi, rdx, rcx, r8, r9]
                                    .drain(0..function_type.params.len())
                                    .collect();
                            for param in function_type.params.iter() {
                                match param {
                                    Type::I64 | Type::I32 => match integer_order.pop_back() {
                                        Some(reg) => assembler.push(reg)?,
                                        None => {}
                                    },
                                    _ => todo!(),
                                }
                            }
                            for op in rd.into_iter() {
                                let op = op?;
                                instructions::handle_instruction(
                                    &mut assembler,
                                    &mut got,
                                    &mut ils,
                                    &mut function_typedefs,
                                    &mut function_types,
                                    op,
                                )?;
                            }
                            let mut integer_order = VecDeque::from([rax, rdx]);
                            for ret in function_type.returns.iter() {
                                match ret {
                                    Type::I64 | Type::I32 => match integer_order.pop_front() {
                                        Some(reg) => assembler.pop(reg)?,
                                        None => (),
                                    },
                                    _ => todo!(),
                                }
                            }
                            assembler.push(r11)?;
                            assembler.ret()?;
                            function_body_index += 1;
                        }
                        _ => (),
                    }
                    data = &data[consumed..];
                    eof = data.len() == 0;
                }
                _ => (),
            }
        }
        Ok(module.assembled(assembler.assemble(0)?))
    }
}

#[cfg(test)]
pub mod testing;

#[cfg(test)]
mod tests;
