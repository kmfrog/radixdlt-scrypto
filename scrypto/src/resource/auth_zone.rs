use sbor::rust::collections::BTreeSet;
use sbor::rust::fmt::Debug;
use sbor::rust::vec::Vec;
use sbor::*;

use crate::engine::utils::*;
use crate::engine::{api::*, types::*};

use crate::math::Decimal;
use crate::resource::*;

#[derive(Debug, TypeId, Encode, Decode)]
pub struct AuthZonePopInvocation {
    pub receiver: AuthZoneId,
}

impl ScryptoNativeInvocation for AuthZonePopInvocation {
    type Output = scrypto::resource::Proof;
}

impl Into<NativeFnInvocation> for AuthZonePopInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::AuthZone(
            AuthZoneMethodInvocation::Pop(self),
        ))
    }
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct AuthZonePushInvocation {
    pub receiver: AuthZoneId,
    pub proof: Proof,
}

impl ScryptoNativeInvocation for AuthZonePushInvocation {
    type Output = ();
}

impl Into<NativeFnInvocation> for AuthZonePushInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::AuthZone(
            AuthZoneMethodInvocation::Push(self),
        ))
    }
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct AuthZoneCreateProofInvocation {
    pub receiver: AuthZoneId,
    pub resource_address: ResourceAddress,
}

impl ScryptoNativeInvocation for AuthZoneCreateProofInvocation {
    type Output = Proof;
}

impl Into<NativeFnInvocation> for AuthZoneCreateProofInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::AuthZone(
            AuthZoneMethodInvocation::CreateProof(self),
        ))
    }
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct AuthZoneCreateProofByAmountInvocation {
    pub receiver: AuthZoneId,
    pub amount: Decimal,
    pub resource_address: ResourceAddress,
}

impl ScryptoNativeInvocation for AuthZoneCreateProofByAmountInvocation {
    type Output = Proof;
}

impl Into<NativeFnInvocation> for AuthZoneCreateProofByAmountInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::AuthZone(
            AuthZoneMethodInvocation::CreateProofByAmount(self),
        ))
    }
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct AuthZoneCreateProofByIdsInvocation {
    pub receiver: AuthZoneId,
    pub ids: BTreeSet<NonFungibleId>,
    pub resource_address: ResourceAddress,
}

impl ScryptoNativeInvocation for AuthZoneCreateProofByIdsInvocation {
    type Output = Proof;
}

impl Into<NativeFnInvocation> for AuthZoneCreateProofByIdsInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::AuthZone(
            AuthZoneMethodInvocation::CreateProofByIds(self),
        ))
    }
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct AuthZoneClearInvocation {
    pub receiver: AuthZoneId,
}

impl ScryptoNativeInvocation for AuthZoneClearInvocation {
    type Output = ();
}

impl Into<NativeFnInvocation> for AuthZoneClearInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::AuthZone(
            AuthZoneMethodInvocation::Clear(self),
        ))
    }
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct AuthZoneDrainInvocation {
    pub receiver: AuthZoneId,
}

impl ScryptoNativeInvocation for AuthZoneDrainInvocation {
    type Output = Vec<scrypto::resource::Proof>;
}

impl Into<NativeFnInvocation> for AuthZoneDrainInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::AuthZone(
            AuthZoneMethodInvocation::Drain(self),
        ))
    }
}

/// Represents the auth zone, which is used by system for checking
/// if this component is allowed to
///
/// 1. Call methods on another component;
/// 2. Access resource system.
pub struct ComponentAuthZone {}

impl ComponentAuthZone {
    #[cfg(target_arch = "wasm32")]
    pub fn pop() -> Proof {
        Self::sys_pop(&mut Syscalls).unwrap()
    }

    pub fn sys_pop<Y, E: Debug + TypeId + Decode>(sys_calls: &mut Y) -> Result<Proof, E>
    where
        Y: ScryptoSyscalls<E> + SysInvokable<AuthZonePopInvocation, E>,
    {
        let owned_node_ids = sys_calls.sys_get_visible_nodes()?;
        let node_id = owned_node_ids
            .into_iter()
            .find(|n| matches!(n, RENodeId::AuthZoneStack(..)))
            .expect("AuthZone does not exist");
        sys_calls.sys_invoke(AuthZonePopInvocation {
            receiver: node_id.into(),
        })
    }

    #[cfg(target_arch = "wasm32")]
    pub fn create_proof(resource_address: ResourceAddress) -> Proof {
        Self::sys_create_proof(resource_address, &mut Syscalls).unwrap()
    }

    pub fn sys_create_proof<Y, E: Debug + TypeId + Decode>(
        resource_address: ResourceAddress,
        sys_calls: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: ScryptoSyscalls<E> + SysInvokable<AuthZoneCreateProofInvocation, E>,
    {
        let owned_node_ids = sys_calls.sys_get_visible_nodes()?;
        let node_id = owned_node_ids
            .into_iter()
            .find(|n| matches!(n, RENodeId::AuthZoneStack(..)))
            .expect("AuthZone does not exist");
        sys_calls.sys_invoke(AuthZoneCreateProofInvocation {
            receiver: node_id.into(),
            resource_address,
        })
    }

    #[cfg(target_arch = "wasm32")]
    pub fn create_proof_by_amount(amount: Decimal, resource_address: ResourceAddress) -> Proof {
        Self::sys_create_proof_by_amount(amount, resource_address, &mut Syscalls).unwrap()
    }

    pub fn sys_create_proof_by_amount<Y, E: Debug + TypeId + Decode>(
        amount: Decimal,
        resource_address: ResourceAddress,
        sys_calls: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: ScryptoSyscalls<E> + SysInvokable<AuthZoneCreateProofByAmountInvocation, E>,
    {
        let owned_node_ids = sys_calls.sys_get_visible_nodes()?;
        let node_id = owned_node_ids
            .into_iter()
            .find(|n| matches!(n, RENodeId::AuthZoneStack(..)))
            .expect("AuthZone does not exist");
        sys_calls.sys_invoke(AuthZoneCreateProofByAmountInvocation {
            receiver: node_id.into(),
            amount,
            resource_address,
        })
    }

    #[cfg(target_arch = "wasm32")]
    pub fn create_proof_by_ids(
        ids: &BTreeSet<NonFungibleId>,
        resource_address: ResourceAddress,
    ) -> Proof {
        Self::sys_create_proof_by_ids(ids, resource_address, &mut Syscalls).unwrap()
    }

    pub fn sys_create_proof_by_ids<Y, E: Debug + TypeId + Decode>(
        ids: &BTreeSet<NonFungibleId>,
        resource_address: ResourceAddress,
        sys_calls: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: ScryptoSyscalls<E> + SysInvokable<AuthZoneCreateProofByIdsInvocation, E>,
    {
        let owned_node_ids = sys_calls.sys_get_visible_nodes()?;
        let node_id = owned_node_ids
            .into_iter()
            .find(|n| matches!(n, RENodeId::AuthZoneStack(..)))
            .expect("AuthZone does not exist");
        sys_calls.sys_invoke(AuthZoneCreateProofByIdsInvocation {
            receiver: node_id.into(),
            ids: ids.clone(),
            resource_address,
        })
    }

    #[cfg(target_arch = "wasm32")]
    pub fn push<P: Into<Proof>>(proof: P) {
        Self::sys_push(proof, &mut Syscalls).unwrap()
    }

    pub fn sys_push<P: Into<Proof>, Y, E: Debug + TypeId + Decode>(
        proof: P,
        sys_calls: &mut Y,
    ) -> Result<(), E>
    where
        Y: ScryptoSyscalls<E> + SysInvokable<AuthZonePushInvocation, E>,
    {
        let owned_node_ids = sys_calls.sys_get_visible_nodes()?;
        let node_id = owned_node_ids
            .into_iter()
            .find(|n| matches!(n, RENodeId::AuthZoneStack(..)))
            .expect("AuthZone does not exist");

        let proof: Proof = proof.into();

        sys_calls.sys_invoke(AuthZonePushInvocation {
            receiver: node_id.into(),
            proof,
        })
    }
}
