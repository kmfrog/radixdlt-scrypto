use crate::engine::{
    ApplicationError, CallFrameUpdate, InvokableNative, LockFlags, NativeExecutable,
    NativeInvocation, NativeInvocationInfo, RENode, RuntimeError, SystemApi,
};
use crate::fee::{FeeReserve, FeeReserveError};
use crate::model::{
    BucketSubstate, InvokeError, ProofError, ResourceContainerId, ResourceOperationError,
};
use crate::types::*;

#[derive(Debug, Clone, PartialEq, Eq, TypeId, Encode, Decode)]
pub enum VaultError {
    InvalidRequestData(DecodeError),
    ResourceOperationError(ResourceOperationError),
    CouldNotCreateBucket,
    CouldNotTakeBucket,
    ProofError(ProofError),
    CouldNotCreateProof,
    LockFeeNotRadixToken,
    LockFeeInsufficientBalance,
    LockFeeRepayFailure(FeeReserveError),
}

impl NativeExecutable for VaultTakeInput {
    type NativeOutput = scrypto::resource::Bucket;

    fn execute<'s, 'a, Y, R>(
        input: Self,
        system_api: &mut Y,
    ) -> Result<(scrypto::resource::Bucket, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi<'s, R> + InvokableNative<'a>,
        R: FeeReserve,
    {
        let node_id = RENodeId::Vault(input.vault_id);
        let offset = SubstateOffset::Vault(VaultOffset::Vault);
        let vault_handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let container = {
            let mut substate_mut = system_api.get_ref_mut(vault_handle)?;
            let vault = substate_mut.vault();
            vault.take(input.amount).map_err(|e| match e {
                InvokeError::Error(e) => {
                    RuntimeError::ApplicationError(ApplicationError::VaultError(e))
                }
                InvokeError::Downstream(runtime_error) => runtime_error,
            })?
        };

        let bucket_id = system_api
            .create_node(RENode::Bucket(BucketSubstate::new(container)))?
            .into();

        Ok((
            scrypto::resource::Bucket(bucket_id),
            CallFrameUpdate::move_node(RENodeId::Bucket(bucket_id)),
        ))
    }
}

impl NativeInvocation for VaultTakeInput {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::Vault(VaultMethod::Take),
            RENodeId::Vault(self.vault_id),
            CallFrameUpdate::empty(),
        )
    }
}

impl NativeExecutable for VaultPutInput {
    type NativeOutput = ();

    fn execute<'s, 'a, Y, R>(
        input: Self,
        system_api: &mut Y,
    ) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi<'s, R> + InvokableNative<'a>,
        R: FeeReserve,
    {
        let node_id = RENodeId::Vault(input.vault_id);
        let offset = SubstateOffset::Vault(VaultOffset::Vault);
        let vault_handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let bucket = system_api
            .drop_node(RENodeId::Bucket(input.bucket.0))?
            .into();

        let mut substate_mut = system_api.get_ref_mut(vault_handle)?;
        let vault = substate_mut.vault();
        vault.put(bucket).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::VaultError(
                VaultError::ResourceOperationError(e),
            ))
        })?;

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl NativeInvocation for VaultPutInput {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::Vault(VaultMethod::Put),
            RENodeId::Vault(self.vault_id),
            CallFrameUpdate::move_node(RENodeId::Bucket(self.bucket.0)),
        )
    }
}

impl NativeExecutable for VaultLockFeeInput {
    type NativeOutput = ();

    fn execute<'s, 'a, Y, R>(
        input: Self,
        system_api: &mut Y,
    ) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi<'s, R> + InvokableNative<'a>,
        R: FeeReserve,
    {
        let node_id = RENodeId::Vault(input.vault_id);
        let offset = SubstateOffset::Vault(VaultOffset::Vault);
        let vault_handle = system_api.lock_substate(
            node_id,
            offset,
            LockFlags::MUTABLE | LockFlags::UNMODIFIED_BASE | LockFlags::FORCE_WRITE,
        )?;

        let fee = {
            let mut substate_mut = system_api.get_ref_mut(vault_handle)?;
            let vault = substate_mut.vault();

            // Check resource and take amount
            if vault.resource_address() != RADIX_TOKEN {
                return Err(RuntimeError::ApplicationError(
                    ApplicationError::VaultError(VaultError::LockFeeNotRadixToken),
                ));
            }

            // Take fee from the vault
            vault.take(input.amount).map_err(|_| {
                RuntimeError::ApplicationError(ApplicationError::VaultError(
                    VaultError::LockFeeInsufficientBalance,
                ))
            })?
        };

        // Refill fee reserve
        let changes = system_api.lock_fee(input.vault_id, fee, input.contingent)?;

        // Return changes
        {
            let mut substate_mut = system_api.get_ref_mut(vault_handle)?;
            let vault = substate_mut.vault();
            vault
                .borrow_resource_mut()
                .put(changes)
                .expect("Failed to return fee changes to a locking-fee vault");
        }

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl NativeInvocation for VaultLockFeeInput {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::Vault(VaultMethod::LockFee),
            RENodeId::Vault(self.vault_id),
            CallFrameUpdate::empty(),
        )
    }
}

impl NativeExecutable for VaultTakeNonFungiblesInput {
    type NativeOutput = scrypto::resource::Bucket;

    fn execute<'s, 'a, Y, R>(
        input: Self,
        system_api: &mut Y,
    ) -> Result<(scrypto::resource::Bucket, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi<'s, R> + InvokableNative<'a>,
        R: FeeReserve,
    {
        let node_id = RENodeId::Vault(input.vault_id);
        let offset = SubstateOffset::Vault(VaultOffset::Vault);
        let vault_handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let container = {
            let mut substate_mut = system_api.get_ref_mut(vault_handle)?;
            let vault = substate_mut.vault();
            vault
                .take_non_fungibles(&input.non_fungible_ids)
                .map_err(|e| match e {
                    InvokeError::Error(e) => {
                        RuntimeError::ApplicationError(ApplicationError::VaultError(e))
                    }
                    InvokeError::Downstream(runtime_error) => runtime_error,
                })?
        };

        let bucket_id = system_api
            .create_node(RENode::Bucket(BucketSubstate::new(container)))?
            .into();

        Ok((
            scrypto::resource::Bucket(bucket_id),
            CallFrameUpdate::move_node(RENodeId::Bucket(bucket_id)),
        ))
    }
}

impl NativeInvocation for VaultTakeNonFungiblesInput {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::Vault(VaultMethod::TakeNonFungibles),
            RENodeId::Vault(self.vault_id),
            CallFrameUpdate::empty(),
        )
    }
}

impl NativeExecutable for VaultGetAmountInput {
    type NativeOutput = Decimal;

    fn execute<'s, 'a, Y, R>(
        input: Self,
        system_api: &mut Y,
    ) -> Result<(Decimal, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi<'s, R> + InvokableNative<'a>,
        R: FeeReserve,
    {
        let node_id = RENodeId::Vault(input.vault_id);
        let offset = SubstateOffset::Vault(VaultOffset::Vault);
        let vault_handle = system_api.lock_substate(node_id, offset, LockFlags::read_only())?;

        let substate_ref = system_api.get_ref(vault_handle)?;
        let vault = substate_ref.vault();
        let amount = vault.total_amount();

        Ok((amount, CallFrameUpdate::empty()))
    }
}

impl NativeInvocation for VaultGetAmountInput {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::Vault(VaultMethod::GetAmount),
            RENodeId::Vault(self.vault_id),
            CallFrameUpdate::empty(),
        )
    }
}

impl NativeExecutable for VaultGetResourceAddressInput {
    type NativeOutput = ResourceAddress;

    fn execute<'s, 'a, Y, R>(
        input: Self,
        system_api: &mut Y,
    ) -> Result<(ResourceAddress, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi<'s, R> + InvokableNative<'a>,
        R: FeeReserve,
    {
        let node_id = RENodeId::Vault(input.vault_id);
        let offset = SubstateOffset::Vault(VaultOffset::Vault);
        let vault_handle = system_api.lock_substate(node_id, offset, LockFlags::read_only())?;

        let substate_ref = system_api.get_ref(vault_handle)?;
        let vault = substate_ref.vault();
        let resource_address = vault.resource_address();

        Ok((
            resource_address,
            CallFrameUpdate::copy_ref(RENodeId::Global(GlobalAddress::Resource(resource_address))),
        ))
    }
}

impl NativeInvocation for VaultGetResourceAddressInput {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::Vault(VaultMethod::GetResourceAddress),
            RENodeId::Vault(self.vault_id),
            CallFrameUpdate::empty(),
        )
    }
}

impl NativeExecutable for VaultGetNonFungibleIdsInput {
    type NativeOutput = BTreeSet<NonFungibleId>;

    fn execute<'s, 'a, Y, R>(
        input: Self,
        system_api: &mut Y,
    ) -> Result<(BTreeSet<NonFungibleId>, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi<'s, R> + InvokableNative<'a>,
        R: FeeReserve,
    {
        let node_id = RENodeId::Vault(input.vault_id);
        let offset = SubstateOffset::Vault(VaultOffset::Vault);
        let vault_handle = system_api.lock_substate(node_id, offset, LockFlags::read_only())?;

        let substate_ref = system_api.get_ref(vault_handle)?;
        let vault = substate_ref.vault();
        let ids = vault.total_ids().map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::VaultError(
                VaultError::ResourceOperationError(e),
            ))
        })?;

        Ok((ids, CallFrameUpdate::empty()))
    }
}

impl NativeInvocation for VaultGetNonFungibleIdsInput {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::Vault(VaultMethod::GetNonFungibleIds),
            RENodeId::Vault(self.vault_id),
            CallFrameUpdate::empty(),
        )
    }
}

impl NativeExecutable for VaultCreateProofInput {
    type NativeOutput = scrypto::resource::Proof;

    fn execute<'s, 'a, Y, R>(
        input: Self,
        system_api: &mut Y,
    ) -> Result<(scrypto::resource::Proof, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi<'s, R> + InvokableNative<'a>,
        R: FeeReserve,
    {
        let node_id = RENodeId::Vault(input.vault_id);
        let offset = SubstateOffset::Vault(VaultOffset::Vault);
        let vault_handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let proof = {
            let mut substate_mut = system_api.get_ref_mut(vault_handle)?;
            let vault = substate_mut.vault();
            vault
                .create_proof(ResourceContainerId::Vault(input.vault_id))
                .map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::VaultError(
                        VaultError::ProofError(e),
                    ))
                })?
        };
        let proof_id = system_api.create_node(RENode::Proof(proof))?.into();

        Ok((
            scrypto::resource::Proof(proof_id),
            CallFrameUpdate::move_node(RENodeId::Proof(proof_id)),
        ))
    }
}

impl NativeInvocation for VaultCreateProofInput {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::Vault(VaultMethod::CreateProof),
            RENodeId::Vault(self.vault_id),
            CallFrameUpdate::empty(),
        )
    }
}

impl NativeExecutable for VaultCreateProofByAmountInput {
    type NativeOutput = scrypto::resource::Proof;

    fn execute<'s, 'a, Y, R>(
        input: Self,
        system_api: &mut Y,
    ) -> Result<(scrypto::resource::Proof, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi<'s, R> + InvokableNative<'a>,
        R: FeeReserve,
    {
        let node_id = RENodeId::Vault(input.vault_id);
        let offset = SubstateOffset::Vault(VaultOffset::Vault);
        let vault_handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let proof = {
            let mut substate_mut = system_api.get_ref_mut(vault_handle)?;
            let vault = substate_mut.vault();
            vault
                .create_proof_by_amount(input.amount, ResourceContainerId::Vault(input.vault_id))
                .map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::VaultError(
                        VaultError::ProofError(e),
                    ))
                })?
        };

        let proof_id = system_api.create_node(RENode::Proof(proof))?.into();

        Ok((
            scrypto::resource::Proof(proof_id),
            CallFrameUpdate::move_node(RENodeId::Proof(proof_id)),
        ))
    }
}

impl NativeInvocation for VaultCreateProofByAmountInput {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::Vault(VaultMethod::CreateProofByAmount),
            RENodeId::Vault(self.vault_id),
            CallFrameUpdate::empty(),
        )
    }
}

impl NativeExecutable for VaultCreateProofByIdsInput {
    type NativeOutput = scrypto::resource::Proof;

    fn execute<'s, 'a, Y, R>(
        input: Self,
        system_api: &mut Y,
    ) -> Result<(scrypto::resource::Proof, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi<'s, R> + InvokableNative<'a>,
        R: FeeReserve,
    {
        let node_id = RENodeId::Vault(input.vault_id);
        let offset = SubstateOffset::Vault(VaultOffset::Vault);
        let vault_handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let proof = {
            let mut substate_mut = system_api.get_ref_mut(vault_handle)?;
            let vault = substate_mut.vault();
            vault
                .create_proof_by_ids(&input.ids, ResourceContainerId::Vault(input.vault_id))
                .map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::VaultError(
                        VaultError::ProofError(e),
                    ))
                })?
        };

        let proof_id = system_api.create_node(RENode::Proof(proof))?.into();

        Ok((
            scrypto::resource::Proof(proof_id),
            CallFrameUpdate::move_node(RENodeId::Proof(proof_id)),
        ))
    }
}

impl NativeInvocation for VaultCreateProofByIdsInput {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::Vault(VaultMethod::CreateProofByIds),
            RENodeId::Vault(self.vault_id),
            CallFrameUpdate::empty(),
        )
    }
}
