use crate::engine::{
    AuthModule, CallFrameUpdate, Invokable, InvokableNative, LockFlags, NativeExecutable,
    NativeInvocation, NativeInvocationInfo, REActor, RENode, ResolvedReceiver, RuntimeError,
    SystemApi,
};
use crate::fee::FeeReserve;
use crate::model::{
    EpochManagerSubstate, GlobalAddressSubstate, HardAuthRule, HardProofRule,
    HardResourceOrNonFungible, MethodAuthorization,
};
use crate::types::*;

#[derive(Debug, Clone, Eq, PartialEq, TypeId, Encode, Decode)]
pub enum EpochManagerError {
    InvalidRequestData(DecodeError),
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct EpochManager {
    pub info: EpochManagerSubstate,
}

impl NativeExecutable for EpochManagerCreateInput {
    type NativeOutput = SystemAddress;

    fn execute<'s, 'a, Y, R>(
        _invocation: Self,
        system_api: &mut Y,
    ) -> Result<(SystemAddress, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi<'s, R> + Invokable<ScryptoInvocation> + InvokableNative<'a>,
        R: FeeReserve,
    {
        let node_id =
            system_api.create_node(RENode::EpochManager(EpochManagerSubstate { epoch: 0 }))?;

        let global_node_id = system_api.create_node(RENode::Global(
            GlobalAddressSubstate::System(node_id.into()),
        ))?;

        let system_address: SystemAddress = global_node_id.into();
        let mut node_refs_to_copy = HashSet::new();
        node_refs_to_copy.insert(global_node_id);

        let update = CallFrameUpdate {
            node_refs_to_copy,
            nodes_to_move: vec![],
        };

        Ok((system_address, update))
    }
}

impl NativeInvocation for EpochManagerCreateInput {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Function(
            NativeFunction::EpochManager(EpochManagerFunction::Create),
            CallFrameUpdate::empty(),
        )
    }
}

impl NativeExecutable for EpochManagerGetCurrentEpochInput {
    type NativeOutput = u64;

    fn execute<'s, 'a, Y, R>(
        _input: Self,
        system_api: &mut Y,
    ) -> Result<(u64, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi<'s, R> + InvokableNative<'a>,
        R: FeeReserve,
    {
        // TODO: Remove this hack and get resolved receiver in a better way
        let node_id = match system_api.get_actor() {
            REActor::Method(_, ResolvedReceiver { receiver, .. }) => *receiver,
            _ => panic!("Unexpected"),
        };
        let offset = SubstateOffset::EpochManager(EpochManagerOffset::EpochManager);
        let handle = system_api.lock_substate(node_id, offset, LockFlags::read_only())?;

        let substate_ref = system_api.get_ref(handle)?;
        let system = substate_ref.epoch_manager();

        Ok((system.epoch, CallFrameUpdate::empty()))
    }
}

impl NativeInvocation for EpochManagerGetCurrentEpochInput {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::EpochManager(EpochManagerMethod::GetCurrentEpoch),
            RENodeId::Global(GlobalAddress::System(self.system_address)),
            CallFrameUpdate::empty(),
        )
    }
}

impl NativeExecutable for EpochManagerSetEpochInput {
    type NativeOutput = ();

    fn execute<'s, 'a, Y, R>(
        input: Self,
        system_api: &mut Y,
    ) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi<'s, R> + InvokableNative<'a>,
        R: FeeReserve,
    {
        // TODO: Remove this hack and get resolved receiver in a better way
        let node_id = match system_api.get_actor() {
            REActor::Method(_, ResolvedReceiver { receiver, .. }) => *receiver,
            _ => panic!("Unexpected"),
        };
        let offset = SubstateOffset::EpochManager(EpochManagerOffset::EpochManager);
        let handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let mut substate_mut = system_api.get_ref_mut(handle)?;
        substate_mut.epoch_manager().epoch = input.epoch;

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl NativeInvocation for EpochManagerSetEpochInput {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::EpochManager(EpochManagerMethod::SetEpoch),
            RENodeId::Global(GlobalAddress::System(self.system_address)),
            CallFrameUpdate::empty(),
        )
    }
}

impl EpochManager {
    pub fn function_auth(func: &EpochManagerFunction) -> Vec<MethodAuthorization> {
        match func {
            EpochManagerFunction::Create => {
                vec![MethodAuthorization::Protected(HardAuthRule::ProofRule(
                    HardProofRule::Require(HardResourceOrNonFungible::NonFungible(
                        NonFungibleAddress::new(SYSTEM_TOKEN, AuthModule::system_id()),
                    )),
                ))]
            }
        }
    }

    pub fn method_auth(method: &EpochManagerMethod) -> Vec<MethodAuthorization> {
        match method {
            EpochManagerMethod::SetEpoch => {
                vec![MethodAuthorization::Protected(HardAuthRule::ProofRule(
                    HardProofRule::Require(HardResourceOrNonFungible::NonFungible(
                        NonFungibleAddress::new(SYSTEM_TOKEN, AuthModule::supervisor_id()),
                    )),
                ))]
            }
            _ => vec![],
        }
    }
}
