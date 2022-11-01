use crate::engine::errors::KernelError;
use crate::engine::*;
use crate::fee::*;
use crate::model::{ InvokeError, KeyValueStore, RuntimeSubstate, };
use crate::types::*;
use crate::wasm::*;
use scrypto::engine::api::ScryptoSyscalls;

/// A glue between system api (call frame and track abstraction) and WASM.
///
/// Execution is free from a costing perspective, as we assume
/// the system api will bill properly.
pub struct RadixEngineWasmRuntime<'y, 's, 'a, Y, R>
where
    Y: SystemApi<'s, R>
        + ScryptoSyscalls<RuntimeError>
        + Invokable<ScryptoInvocation>
        + InvokableNative<'a>,
    R: FeeReserve,
{
    system_api: &'y mut Y,
    phantom1: PhantomData<R>,
    phantom2: PhantomData<&'s ()>,
    phantom3: PhantomData<&'a ()>,
}

impl<'y, 's, 'a, Y, R> RadixEngineWasmRuntime<'y, 's, 'a, Y, R>
where
    Y: SystemApi<'s, R>
        + ScryptoSyscalls<RuntimeError>
        + Invokable<ScryptoInvocation>
        + InvokableNative<'a>,
    R: FeeReserve,
{
    // TODO: expose API for reading blobs

    // TODO: do we want to allow dynamic creation of blobs?

    // TODO: do we check existence of blobs when being passed as arguments/return?

    pub fn new(system_api: &'y mut Y) -> Self {
        RadixEngineWasmRuntime {
            system_api,
            phantom1: PhantomData,
            phantom2: PhantomData,
            phantom3: PhantomData,
        }
    }

    fn handle_invoke_native_function(
        &mut self,
        native_function: NativeFunction,
        args: Vec<u8>,
    ) -> Result<ScryptoValue, RuntimeError> {
        parse_and_invoke_native_function(native_function, args, self.system_api)
    }

    fn handle_invoke_native_method(
        &mut self,
        native_method: NativeMethod,
        args: Vec<u8>,
    ) -> Result<ScryptoValue, RuntimeError> {
        parse_and_invoke_native_method(native_method, args, self.system_api)
    }

    fn handle_get_transaction_hash(&mut self) -> Result<Hash, RuntimeError> {
        self.system_api.read_transaction_hash()
    }
}

fn encode<T: Encode>(output: T) -> Vec<u8> {
    scrypto_encode(&output)
}

impl<'y, 's, 'a, Y, R> WasmRuntime for RadixEngineWasmRuntime<'y, 's, 'a, Y, R>
where
    Y: SystemApi<'s, R>
        + ScryptoSyscalls<RuntimeError>
        + Invokable<ScryptoInvocation>
        + InvokableNative<'a>,
    R: FeeReserve,
{
    fn main(&mut self, input: ScryptoValue) -> Result<Vec<u8>, InvokeError<WasmError>> {
        let input: RadixEngineInput = scrypto_decode(&input.raw)
            .map_err(|_| InvokeError::Error(WasmError::InvalidRadixEngineInput))?;
        let rtn = match input {
            RadixEngineInput::InvokeScryptoFunction(function_ident, args) => {
                self.system_api.sys_invoke_scrypto_function(function_ident, args)?
            }
            RadixEngineInput::InvokeScryptoMethod(method_ident, args) => {
                self.system_api.sys_invoke_scrypto_method(method_ident, args)?
            }
            RadixEngineInput::InvokeNativeFunction(native_function, args) => {
                self.handle_invoke_native_function(native_function, args).map(|v| v.raw)?
            }
            RadixEngineInput::InvokeNativeMethod(native_method, args) => {
                self.handle_invoke_native_method(native_method, args).map(|v| v.raw)?
            }
            RadixEngineInput::CreateNode(node) => {
                self.system_api.sys_create_node(node).map(encode)?
            },
            RadixEngineInput::GetVisibleNodeIds() => {
                self.system_api.sys_get_visible_nodes().map(encode)?
            },
            RadixEngineInput::DropNode(node_id) => {
                self.system_api.sys_drop_node(node_id).map(encode)?
            },
            RadixEngineInput::LockSubstate(node_id, offset, mutable) => {
                self.system_api.sys_lock_substate(node_id, offset, mutable).map(encode)?
            }
            RadixEngineInput::Read(lock_handle) => {
                self.system_api.sys_read(lock_handle)?
            },
            RadixEngineInput::Write(lock_handle, value) => {
                self.system_api.sys_write(lock_handle, value).map(encode)?
            },
            RadixEngineInput::DropLock(lock_handle) => {
                self.system_api.sys_drop_lock(lock_handle).map(encode)?
            },
            RadixEngineInput::GetActor() => {
                self.system_api.sys_get_actor().map(encode)?
            },
            RadixEngineInput::GetTransactionHash() => {
                self.handle_get_transaction_hash().map(encode)?
            }
            RadixEngineInput::GenerateUuid() => {
                self.system_api.sys_generate_uuid().map(encode)?
            },
            RadixEngineInput::EmitLog(level, message) => {
                self.system_api.sys_emit_log(level, message).map(encode)?
            }
        };

        Ok(rtn)
    }

    fn consume_cost_units(&mut self, n: u32) -> Result<(), InvokeError<WasmError>> {
        self.system_api
            .consume_cost_units(n)
            .map_err(InvokeError::downstream)
    }
}

/// A `Nop` runtime accepts any external function calls by doing nothing and returning void.
pub struct NopWasmRuntime {
    fee_reserve: SystemLoanFeeReserve,
}

impl NopWasmRuntime {
    pub fn new(fee_reserve: SystemLoanFeeReserve) -> Self {
        Self { fee_reserve }
    }
}

impl WasmRuntime for NopWasmRuntime {
    fn main(&mut self, _input: ScryptoValue) -> Result<Vec<u8>, InvokeError<WasmError>> {
        Ok(ScryptoValue::unit().raw)
    }

    fn consume_cost_units(&mut self, n: u32) -> Result<(), InvokeError<WasmError>> {
        self.fee_reserve
            .consume(n, "run_wasm", false)
            .map_err(|e| InvokeError::Error(WasmError::CostingError(e)))
    }
}
