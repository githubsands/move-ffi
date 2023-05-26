use move_binary_format::{
    errors::{VMError, VMResult},
    file_format::{
        AbilitySet, AddressIdentifierIndex, Bytecode, CodeUnit, CompiledModule, FieldDefinition,
        FunctionDefinition, FunctionHandle, FunctionHandleIndex, IdentifierIndex, ModuleHandle,
        ModuleHandleIndex, Signature, SignatureIndex, SignatureToken, StructDefinition,
        StructFieldInformation, StructHandle, StructHandleIndex, TableIndex, TypeSignature,
        Visibility,
    },
};
use move_core_types::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::{ModuleId, StructTag, TypeTag},
    resolver::{ModuleResolver, ResourceResolver},
    value::MoveValue,
};
use move_vm::move_vm::MoveVM;
use move_vm_runtime as move_vm;
use move_vm_types::gas_schedule::GasStatus;
use std::collections::HashMap;

pub fn call_script_function_with_args_ty_args_signers(
    module: CompiledModule,
    function_name: Identifier,
    non_signer_args: Vec<Vec<u8>>,
    ty_args: Vec<TypeTag>,
    signers: Vec<AccountAddress>,
) -> VMResult<()> {
    let move_vm = MoveVM::new(vec![]).unwrap();
    let mut remote_view = RemoteStore::new();
    let id = module.self_id();
    remote_view.add_module(module);
    let mut session = move_vm.new_session(&remote_view);
    let mut gas_status = GasStatus::new_unmetered();
    session.execute_function_bypass_visibility(
        &id,
        function_name.as_ident_str(),
        ty_args,
        combine_signers_and_args(signers, non_signer_args),
        &mut gas_status,
    )?;
    Ok(())
}

pub fn make_module_with_function(
    visibility: Visibility,
    is_entry: bool,
    parameters: Signature,
    return_: Signature,
    type_parameters: Vec<AbilitySet>,
) -> (CompiledModule, Identifier) {
    let function_name = Identifier::new("foo").unwrap();
    let mut signatures = vec![Signature(vec![])];
    let parameters_idx = match signatures
        .iter()
        .enumerate()
        .find(|(_, s)| *s == &parameters)
    {
        Some((idx, _)) => SignatureIndex(idx as TableIndex),
        None => {
            signatures.push(parameters);
            SignatureIndex((signatures.len() - 1) as TableIndex)
        }
    };
    let return_idx = match signatures.iter().enumerate().find(|(_, s)| *s == &return_) {
        Some((idx, _)) => SignatureIndex(idx as TableIndex),
        None => {
            signatures.push(return_);
            SignatureIndex((signatures.len() - 1) as TableIndex)
        }
    };
    let module = CompiledModule {
        version: move_binary_format::file_format_common::VERSION_MAX,
        self_module_handle_idx: ModuleHandleIndex(0),
        module_handles: vec![ModuleHandle {
            address: AddressIdentifierIndex(0),
            name: IdentifierIndex(0),
        }],
        struct_handles: vec![StructHandle {
            module: ModuleHandleIndex(0),
            name: IdentifierIndex(1),
            abilities: AbilitySet::EMPTY,
            type_parameters: vec![],
        }],
        function_handles: vec![FunctionHandle {
            module: ModuleHandleIndex(0),
            name: IdentifierIndex(2),
            parameters: parameters_idx,
            return_: return_idx,
            type_parameters,
        }],
        field_handles: vec![],
        friend_decls: vec![],

        struct_def_instantiations: vec![],
        function_instantiations: vec![],
        field_instantiations: vec![],

        signatures,

        identifiers: vec![
            Identifier::new("M").unwrap(),
            Identifier::new("X").unwrap(),
            function_name.clone(),
        ],
        address_identifiers: vec![AccountAddress::random()],
        constant_pool: vec![],
        metadata: vec![],

        struct_defs: vec![StructDefinition {
            struct_handle: StructHandleIndex(0),
            field_information: StructFieldInformation::Declared(vec![FieldDefinition {
                name: IdentifierIndex(1),
                signature: TypeSignature(SignatureToken::Bool),
            }]),
        }],
        function_defs: vec![FunctionDefinition {
            function: FunctionHandleIndex(0),
            visibility,
            is_entry,
            acquires_global_resources: vec![],
            code: Some(CodeUnit {
                locals: SignatureIndex(0),
                code: vec![Bytecode::LdU64(0), Bytecode::Abort],
            }),
        }],
    };
    (module, function_name)
}

fn combine_signers_and_args(
    signers: Vec<AccountAddress>,
    non_signer_args: Vec<Vec<u8>>,
) -> Vec<Vec<u8>> {
    signers
        .into_iter()
        .map(|s| MoveValue::Signer(s).simple_serialize().unwrap())
        .chain(non_signer_args)
        .collect()
}

struct RemoteStore {
    modules: HashMap<ModuleId, Vec<u8>>,
}

impl RemoteStore {
    fn new() -> Self {
        Self {
            modules: HashMap::new(),
        }
    }

    fn add_module(&mut self, compiled_module: CompiledModule) {
        let id = compiled_module.self_id();
        let mut bytes = vec![];
        compiled_module.serialize(&mut bytes).unwrap();
        self.modules.insert(id, bytes);
    }
}

impl ModuleResolver for RemoteStore {
    type Error = VMError;
    fn get_module(&self, module_id: &ModuleId) -> Result<Option<Vec<u8>>, Self::Error> {
        Ok(self.modules.get(module_id).cloned())
    }
}

impl ResourceResolver for RemoteStore {
    type Error = VMError;

    fn get_resource(
        &self,
        _address: &AccountAddress,
        _tag: &StructTag,
    ) -> Result<Option<Vec<u8>>, Self::Error> {
        Ok(None)
    }
}
